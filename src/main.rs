use std::{
    ops::Range,
    path::{Path, PathBuf, StripPrefixError},
    time::{SystemTime, SystemTimeError},
};

use chrono::{DateTime, NaiveDate, Utc};
use pulldown_cmark::{BrokenLink, CodeBlockKind, CowStr, Event, Options, Tag};
use toml::{value::Datetime, Table};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Template(#[from] Box<handlebars::TemplateError>),
    #[error("{0}")]
    Render(#[from] handlebars::RenderError),
    #[error("{0}")]
    Time(#[from] SystemTimeError),
    #[error("{0}")]
    WalkDir(#[from] walkdir::Error),
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
    #[error("{0}")]
    Prefix(#[from] StripPrefixError),
}

impl From<handlebars::TemplateError> for Error {
    fn from(value: handlebars::TemplateError) -> Self {
        Self::Template(Box::new(value))
    }
}

#[derive(Debug, serde::Serialize)]
struct PageData<'a, T> {
    base_path: &'a str,
    #[serde(flatten)]
    data: T,
}

struct Config {
    base_url: String,
}

fn main() -> Result<(), crate::Error> {
    let mut handlebars = handlebars::Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_templates_directory(".hbs", "templates")?;

    let mut posts = collect_posts()?;

    let cfg = Config {
        base_url: String::from(""),
    };

    posts.sort_by_key(|post| post.created);

    copy_static_files()?;

    posts
        .iter()
        .try_for_each(|post| Post::generate(post, &cfg, &handlebars))?;

    generate_main(&handlebars, &cfg, &posts)?;

    Ok(())
}

fn copy_static_files() -> Result<(), Error> {
    let publish_base: &Path = "publish".as_ref();
    let static_base: &Path = "static".as_ref();
    for file in walkdir::WalkDir::new(static_base)
        .into_iter()
        .filter(|entry| {
            entry.is_err()
                || entry
                    .as_ref()
                    .is_ok_and(|entry| entry.file_type().is_file())
        })
    {
        let file = file?;
        let src_path = file.path();
        let rel_path = src_path.strip_prefix(static_base)?;
        let dest_path = publish_base.join(rel_path);
        std::fs::create_dir_all(dest_path.parent().unwrap())?;
        std::fs::copy(src_path, dest_path)?;
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct PostPreview<'a> {
    link: String,
    title: String,
    keywords: &'a [String],
}

#[derive(Debug, serde::Serialize)]
struct MainPage<'b> {
    recent_posts: &'b [PostPreview<'b>],
}

fn generate_main(
    handlebars: &handlebars::Handlebars,
    cfg: &Config,
    posts: &[Post],
) -> Result<(), crate::Error> {
    let path = "publish/index.html";
    let dir = std::path::Path::parent(path.as_ref()).unwrap();
    std::fs::create_dir_all(dir)?;
    let file = std::fs::File::create(path)?;

    let recent_posts = posts
        .iter()
        .take(10)
        .map(|post| PostPreview {
            link: post.url_path(),
            title: post.title.clone(),
            keywords: &post.keywords,
        })
        .collect::<Vec<_>>();

    handlebars.render_to_write(
        "main",
        &PageData {
            base_path: &cfg.base_url,
            data: MainPage {
                recent_posts: &recent_posts,
            },
        },
        file,
    )?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct Post {
    path: PathBuf,
    title: String,
    body: String,
    metadata: Table,
    created: DateTime<Utc>,
    modified: DateTime<Utc>,
    keywords: Vec<String>,
}

impl Post {
    fn parse(path: &std::path::Path) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path)?;

        fn broken_link(_broken: BrokenLink<'_>) -> Option<(CowStr<'_>, CowStr<'_>)> {
            std::process::exit(1)
        }

        let mut broken_link_callback = broken_link;

        let parser = pulldown_cmark::Parser::new_with_broken_link_callback(
            &content,
            Options::all(),
            Some(&mut broken_link_callback),
        );

        let mut peekable_parser = parser.into_offset_iter().peekable();

        let mut metadata = None;

        if let Some((Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(fence))), _)) =
            peekable_parser.peek()
        {
            if fence.as_ref() == "metadata" {
                metadata = Some(collect_metadata(&mut peekable_parser, &content)?);
            }
        }

        let mut body = Vec::new();

        pulldown_cmark::html::write_html(&mut body, peekable_parser.map(|(e, _)| e))?;

        let (title, created, modified, keywords) = if let Some(metadata) = &metadata {
            let title = metadata
                .get("title")
                .and_then(|title| title.as_str())
                .map(|title| title.to_owned());
            let created = metadata
                .get("created")
                .and_then(|datetime| datetime.as_datetime())
                .map(|datetime| datetime.to_owned());
            let modified = metadata
                .get("modified")
                .and_then(|datetime| datetime.as_datetime())
                .map(|datetime| datetime.to_owned());
            let keywords = metadata
                .get("keywords")
                .and_then(|keywords| keywords.as_array())
                .map(|keywords| {
                    keywords
                        .iter()
                        .filter_map(|keyword| keyword.as_str())
                        .map(|keyword| keyword.to_owned())
                        .collect::<Vec<_>>()
                });
            (title, created, modified, keywords)
        } else {
            (None, None, None, None)
        };

        let title = title.unwrap_or_else(|| {
            path.file_stem()
                .map(|file_stem| file_stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| String::from("Title not found!"))
        });

        let file_metadata = path.metadata()?;

        let created = if let Some(created) = created {
            convert_toml_time(created)
        } else {
            convert_sys_time(file_metadata.created()?)?
        };

        let modified = if let Some(modified) = modified {
            convert_toml_time(modified)
        } else {
            convert_sys_time(file_metadata.created()?)?
        };

        Ok(Self {
            path: path.to_owned(),
            title,
            body: String::from_utf8_lossy(&body).into_owned(),
            metadata: metadata.unwrap_or_default(),
            created,
            modified,
            keywords: keywords.unwrap_or_default(),
        })
    }

    fn url_path(&self) -> String {
        format!(
            "posts/{}/{}.html",
            self.created.format(""),
            self.title
                .replace(' ', "-")
                .to_lowercase()
                .chars()
                .filter(|char| char.is_ascii_graphic())
                .collect::<String>()
        )
    }

    fn generate(
        &self,
        cfg: &Config,
        handlebars: &handlebars::Handlebars,
    ) -> Result<(), crate::Error> {
        let path = format!("publish/{}", self.url_path());
        let dir = std::path::Path::parent(path.as_ref()).unwrap();
        std::fs::create_dir_all(dir)?;
        let file = std::fs::File::create(path)?;
        handlebars.render_to_write(
            "post",
            &PageData {
                base_path: &cfg.base_url,
                data: self,
            },
            file,
        )?;
        Ok(())
    }
}

fn convert_toml_time(toml_time: Datetime) -> DateTime<Utc> {
    let date = toml_time.date.unwrap();
    let time = toml_time.time.unwrap();
    NaiveDate::from_ymd_opt(date.year as i32, date.month as u32, date.day as u32)
        .unwrap()
        .and_hms_opt(time.hour as u32, time.minute as u32, time.second as u32)
        .unwrap()
        .and_utc()
}

fn convert_sys_time(sys_time: SystemTime) -> Result<DateTime<Utc>, SystemTimeError> {
    let since_unix = sys_time.duration_since(SystemTime::UNIX_EPOCH)?;
    Ok(DateTime::from_timestamp(since_unix.as_secs() as i64, since_unix.subsec_nanos()).unwrap())
}

fn collect_metadata<'a, I: Iterator<Item = (Event<'a>, Range<usize>)>>(
    peekable_parser: &mut I,
    content: &str,
) -> Result<Table, toml::de::Error> {
    let mut depth = 0u32;

    let mut start = usize::MAX;
    let mut end = 0;

    for (item, offset) in peekable_parser {
        match item {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {
                if depth != 0 {
                    start = start.min(offset.start);
                    end = end.max(offset.end);
                }
            }
        }
    }

    toml::from_str(&content[start..end])
}

fn collect_posts() -> Result<Vec<Post>, Error> {
    let walk_posts = walkdir::WalkDir::new("pages/posts")
        .sort_by_file_name()
        .into_iter()
        .filter(|entry| {
            entry.is_err()
                || entry
                    .as_ref()
                    .is_ok_and(|entry| entry.file_type().is_file())
        });
    let mut posts = vec![];

    for entry in walk_posts {
        posts.push(Post::parse(entry?.path())?)
    }

    Ok(posts)
}
