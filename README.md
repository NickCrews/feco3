# FECo3

A .FEC file parser in rust, with python bindings

Still in alpha.

## Related projects

Please open an issue or PR if you'd like to add or edit this list.

- [FECfile](https://github.com/esonderegger/fecfile)
  Fairly well maintained parser in python
- [FastFEC](https://github.com/washingtonpost/FastFEC)
  A FEC file parser in C.
- [fec-loader](https://github.com/PublicI/fec-loader)
  Node.js tools and CLI to discover, convert and load raw FEC filings into a database.
- [Fech](https://github.com/dwillis/Fech)
  Ruby downloader and parser. Moderately recently maintained?
- [fech-sources](https://github.com/dwillis/fech-sources)
  Schema definitions for the various line codes. Used by Fech and some other parsers.
- [nyt-pyfec](https://github.com/newsdev/nyt-pyfec)
  Old, unmaintained python parser
- [fec2json](https://github.com/newsdev/fec2json)
  Not complete parser written in python

## Bibliography

Parsing FEC files is an obscure, poorly documented task.

The FEC publishes technical documentation for the files.

1. Go to https://www.fec.gov/data/browse-data/?tab=bulk-data
2. Expand the "Electronically filed reports (.fec files)" section.
3. Click the download link.
4. Do the same for the "Paper filed reports (.fec files)" section.
