# Targets
- Data scientists and MLEs working in NLP
- Flatten the structure of the objects to make them act like the row of database

# Features
- Statistics extractions: Extract statistics from a file-backed dataset
- Recommendations: Recommend some tranformation for data, such as normalisation,
  string to number, date to number, etc.
- Sampling: Sample some exemples to view in the terminal

# Glossary
- File-dataset: Dataset backed by a file on disk

# Architecture
## Lib
Use the arrow types internally. Wrap it inside a small struct.
## Reading
### Supported Formats
- JSON -> Read the whole file first ?
- JSONL
- CSV
- Parquet

Use the arrow.rs crate to read 
## Pipeline

## Config
## Caching

