```metadata
title =  "A critique of 'An INI critique of TOML'"
created =  2023-10-02T19:43:00
modified = 2023-10-02T19:43:00
keywords = ["critique", "TOML", "INI"]
```

A critique of '[An INI critique of TOML][critique]'
===================================================

I was just reading to the [latest comment][cabal-comment] on an Issue of Haskell's Cabal Project.
The issue is about switching from the `.cabal` config format to a standard config format such as TOML.
That comment references '[An INI critique of TOML][critique]',
which find my self strongly disagreeing with to the point that I think it might be sarcastic.
As such I want to give my points regarding that critique.
I am probably biased as I have been using TOML for
multiple project already and I have come to prefer it over
JSON, Yaml, INI and god forbid Xml.

The critique is divided into sections which I will replicate here,
the preamble section refers to the text before the first section,
and section 0 is a point that appears to several sections.

Preamble
--------

Here in the preamble the author says that all INI dialects are well defined,
 as one has to just look at the applications parser implementation.
 How is that a well defined config format,
  if one has to look at a given implementation out many to know what is valid and how it is interpreted.

What I need independent of config format is documentation of the accepted/required keys and
the definition of valid values.
What toml gives me as a config writer is
a standard syntax to write values of specific primitive types (number, string, boolean, date),
as well as a standard syntax to write composite types.

\0. Confusion between how TOML interprets a file and how a program interprets the resulting TOML
-----------------------------------------------------------------------------------------------

I think the Author confuses parsing TOML into an abstract TOML Document structure
and the application interpreting said structure.

I consider the TOML parsing as required by the specification to end at the abstract TOML Document structure.
This structure is then assigned meaning by the application for which it is valid to
ignore distinctions made by the underlying TOML Format.

The author appears to not make this distinction and as such come to a, in my opinion,
false conclusion in many of the following aspects.

Some decisions in TOML make sense when you have such a split.
A TOML config should in the abstract be parsable by any TOML implementation.
You need sufficiently useful primitives if aou don't want to fallback on having everything be strings.

I think TOML is designed to be abstracted by a library that parses a file into an
abstract representation rather than having each application parse it directly.
For this the more strict nature of TOML makes sense in my opinion.

The application then does not consume a TOML file directly,
but instead an abstract TOML Document type returned by the TOML library.
As a result adding a layer of abstraction between the config file and the application.

\1. Data types
--------------

The author complains about the typed nature of TOML,
and approves of the stringly typed nature of INI.

As someone that prefers strongly typed languages must disagree.
Having a distinction between `89` and `"89"` is meaningful as it conveys semantics,
both to the application and someone handling the config file.
Whether the application finds this semantic difference meaningful is a different thing.
If the program finds such semantics distinction meaningful and
the user used a type considered invalid,
 it should rightfully be able to raise this as an error to the user for them to fix.
If the application doesn't care it can always convert both to whichever type it uses internally.
As such TOML preserves semantics so that the application can decide whether it cares.

Additionally it lets the toml library handle some parsing for us while supporting
nice things.
For example toml supports hexadecimal (0xff), octal (0o77) and binary (0b11) number integer literals,
exponents (-5e-2), infinity (+inf, inf, -inf) and NaN (nan, +nan, -nan) for floats.
Both integers and floats support separators,
 so that you can for example write a long Billion as 1_000_000_000_000,
 instead of 1000000000000.
It is the TOML parsers job to parse all of these representations the respective type,
instead of having each application deal with parsing strings that could be in any of these formats,
the application gets the already parsed value.

Bringing up enums is also not convincing as no abstract format can anticipate every
possible enum and the set of valid values for each application.
Falling back to using a string and having the application handle decoding from string is a valid strategy,
if the set of expected values is application defined.

\2. Quotes in values
--------------------

Here the author complains about the absence of unquoted strings,
i.e. all strings need to be surrounded by quotes.

If you followed me on the previous points some that you should agree that quoted strings are necessary,
 as we need to distinguish `0xff` from `"0xff"`, `true` from `"true"` and `[ 89 ]` from `"[ 89 ]".`
It is also necessary to make sure leading and trailing whitespace in unambiguous,
Is

```ini
some_key=text
```

the same as

```ini
some_key = text
```

maybe, maybe not with quotes it is directly clear what is part of the string and what is not.
I accidentally had a 1.70 truncated to 1.7 in a CI config file,
as I forgot to quote my string when changing a version number from `1.69` to `1.70`,
this could have been avoided if Yaml and/or the application did not perform auto-conversion
and only accepted string for version numbers.

\3. Case sensitivity
--------------------

I think this comes back to my 0th point, whether an application,
treats keys case-sensitive or not is up to the application,
but TOML preserves that difference so that the application can make that choice.

\4. Unicode key names
---------------------

At least for keys not containing whitespace or special characters used by the toml syntax,
I agree with this point.
But I don't consider this more than a nit,
having to add quotes is a minor inconvenience and
in my experience I find it rather uncommon to have non-ASCII keys.
Though I suspect users of alphabets that are not based on the latin alphabet will likely disagree.

\5. Square brackets
-------------------

A big point for square brackets is that they allow using the same delimiter
independent of nesting level,
i.e. I don't need to remember which nesting level uses which delimiter.
Additionally having explicit start/end of list marker means there is no need for explicit line continuation,
 as it is implied by the unfinished list.
I find a backslash at the end of the line to signal line continuation quite ugly.
Also it easy to accidentally have some stray whitespace between the backslash and
the newline resulting in the line continuation not working.

The point regarding one member arrays is again a point where I find this to be a valid semantical distinction:
A string and an array containing just the same string are not the same.

Humans not understanding this distinction should not be hand-writing config files,
they should change the config via an application settings dialog.

Conflating a pointer to a value and a pointer to an array of values should not be
easy to confuse,
but in C a pointer to an array is just a pointer to the arrays first value, muddling the distinction.
Such a flaw of C does not need to exist in TOML.

\6. Array delimiters
--------------------

This is the point where I began seriously suspecting that this is all just a huge joke.

The important part is that the human and application agree about the meaning of a config file.
This is in my experience easier if one adheres to convention over configuration.
Having arbitrary delimiters just flips this on its head while honking a clowns nose.

Yes, IPv4 address is usually written as dot separated decimal literals in the range 0-255,
but I am willing to say that a comma is by far the most prevalent delimiter
for lists and
with brackets to mark the beginning and end pf lists you only need one delimiter
as nesting levels are already clear.

\7. Mixed arrays
----------------

Fist the author complained about the typed nature of TOML and now its not typed enough?
Also is it so hard to come up with the idea that a mixed array can be represented
as an array of a tagged union type of all possible types.
There are only finite types in TOML after all `string`, `integer`, `float`, `date`, `boolean`, `(mixed) array` and  `table`.

Whether the application accepts such a mixed array is again up to the application.

\8. Composite configuration files
---------------------------------

This in part assumes that composition of config files is done by concatenating the
config files content.
I have never wanted to combine configuration files this way.
If an application wants support merging config files,
the right approach in my opinion would be to defined an import mechanism
with proper conflict resolution.
Wild west file concatenation seams inappropriate to me.

\9. Dates
---------

### Why Dates?

- We have well defined Datetime formats
- One standard format for all TOML files, rather than a format for each application
- The TOML library can handle parsing in one place rather than each application rolling their own
- Datetime are common and usually require special handling for modification and comparison
  - especially with proper Timezones handling
- Datetime handling not standard in most programming language standard libraries.

### Paths

Paths differ significantly between Windows and Unix systems.
So I don't think having a special case above string makes sense,
especially as in most languages I know they are just that strings.
Rust being one of the few exceptions I know that have a special type.
Usually paths in configs are either a complete path or a base path
and as such barely require modification by the application.
Languages that support Filesystem operations usually
include Path handling functions.

### Usernames

What is and is not a valid Username is not well defined
and highly application specific as such it is not feasible to
have a specific Username type in an abstract config format.
Also Usernames akt in most cases as a simple string key and are otherwise
unused computationally.

### Email Address

I think its rare to need to perform special handling on E-Mail Addresses
beyond passing it to the function that will be sending emails.

\10. Empty key names
--------------------

Why should the empty string be special? Keys are basically just strings.

\11. Arrays of tables
--------------------

Not everything needs a name.
Having a list of objects or in TOML terms an array of tables is a reasonable thing in my opinion.
If you want to name the entries add a name key and assign it whichever name you like,
in my experience most programs simply ignore unexpected keys.
Alternatively add a comment.
I find the example of using the IP as part of the path especially bad,
as now I need to dissect the path to get the IP.

This is again also a semantics difference between a `HashMap<String, Object>` and a `List<Object>`.
If the object is named is it externally tagged with the name or is it internally tagged with the name,
neither option is right or wrong, it depends on the use-case.
As the author has demonstrated TOML supports both use-cases.
I personally prefer the internally tagged List of Objects for configuration files.

\12. Lack of support for implicit keys
--------------------------------------

I find neither example convincing, I would find it more convincing
if it where treated as a `HashMap<String, Option<Value>>` where one
can differentiate between an absent key, a key with a absent value and a key with a value.
That an implementation decides,
that a key with an absent value is equivalent to a key with the value `true`,
should then be a decision made by the application,
not the config format.
TOML would than analogue represent itself as a `HashMap<String, Value>`,
here a key with an absent key is obviously not valid and I prefer this.
If you define a key typing a few characters more to define its value is not too much to ask.

\13. Inline tables must remain... inline
----------------------------------------

I agree ith this one.
To make it worse an inline table is allowed to span multiple lines if the line breaks
are in values that permit line breaks, i.e. arrays and multiline strings.

\14. Incompatibility
--------------------

Most config formats are incompatible with each other.
Also, I think they meant to say that Yaml is a superset of JSON in the Preamble,
not a subset.

\15.Immediacy
-------------

**Microsoft Notepad** is no longer a reasonable editor in my opinion.
I also find TOML easier to edit compared to INI as it is more standardized.

The example even shows this as the TOML does not require a comment to indicate that
`client."hello world"` accepts a list as the current value is already a list.

How would the user know what datatype an array contains in TOML?
If its none empty they can just see it based on the existing value.
Otherwise it is not worse than INI as everything is a string in INT,
at least in TOML we may be able to see an empty list if the key hasn't been omitted.

\16. Genesis
------------

Apparently the author doesn't like the tradeoffs made in TOML compared to INI.
This takes one aspect, that the TOML creator didn't include unquoted strings because he didn't like them,
as the complete genesis of TOML.
I don't think that JSON is a bad starting point type-wise when coming up with a new config format.
A new standart is born from "I don't like the options available" and "I think I can do better,
by learning from what's available", like any other standard out there.
I find in this regard TOML to be an improvement upon INI as it being more strict/opinionated,
results in it being less fragmented.

Regarding that every TOML file could be a JSON file instead,
I would like to state the opposite. Every JSON file should be a TOML file instead.
As the author has already pointed out JSON was devised as a serialization format and
is based on JavaScript object notation as it allowed one to simply eval a JSON string into a JavaScript object,
though that would be ill advised as it would allow injection if arbitrary JS-Code.

Compared to JSON I find TOML significantly more readable and
easier to write correctly by hand.

Compared to Yaml TOML does not suffer from unquoted strings,
especially multiline unquoted strings with their sigils I can never get right,
or significant whitespace.

\17. Against [Postel's law][postels-law] by design
-----------------------------------

I think this is another point that requires comparing different aspects against one another.
I stand with TOML in that I prefer a strict config format that notifies my about mistakes.
Also, I find that usually the distance between manually editing a config file and
running the relevant program is rather short,
so that I am usually expecting such error to be possible.

Already discussed this in point 4. Unicode key names.

While I agree that usually it is better to have only one way to express a given thing.
I disagree that just because there are multiple ways it is necessarily bad.
I find that inline tables can be useful if they are small for example
I mostly use them in the Cargo.toml's `[dependencies]` table,
when I need more than a simple version and
usually an inline table is sufficiently legible that I don't require a full blown table.

I generally prefer an error when something is wring/invalid that having the program
silently guess what I meant. I find this to be one of the greatest problem of
e.g. JavaScript and HTML that try to continue no matter what.

\18. Performance
----------------

Bad example parser and file: <https://github.com/madmurphy/libconfini/issues/19>

Also why is that Config file 50MB?
If that is documentation of all possible options,
it should probably be a separate documentation file.
If it is actual config data, it should probably be a database instead.

\19. Human-friendly vs. human-readable
--------------------------------------

I disagree on the human-readable.ness of JSON.
I find JSON barely human readable just somewhat better than XML,
and at least less ambiguous than Yaml.
In regards to human-writable-ness of JSON, I think it would be
a lot better if it allowed trailing comma in objects and arrays,
as that is my main problem with writing JSON,
though not all JSON parsers are that strict and accept trailing comma.

Regarding readability I agree that INI and TOML are similar in readability.
Though I would argue due to not being stringly types that TOML is easier to write correctly.

\20. Aesthetics
---------------

This section doesn't really contain a point, but I prefer my TOML files
to not be indented except for multiline arrays.

Conclusion
----------

Overall I must disagree with the assessment that TOML should not be used
due to its numerous flaws.
Mostly this appears to be due to me disagreeing with the authors premiss,
leading me to a different conclusion.
Additionally, the author appear to conflate the abstract semantics of TOML with
the semantics that an application layers on top of TOML,
for me this resolves some of the complains the author has with TOML.
I agree that limiting unquoted keys to ascii is unnecessary limiting,
but I disagree that whitespace should be allowed in unquoted keys.

[cabal-comment]: https://github.com/haskell/cabal/issues/7548#issuecomment-1742568148
[critique]: https://github.com/madmurphy/libconfini/wiki/An-INI-critique-of-TOML
[postels-law]: https://en.wikipedia.org/wiki/Robustness_principle
