# Efetch2jsonl

A simple parser to convert from the output from [efetch](https://www.ncbi.nlm.nih.gov/books/NBK179288/) to a more parseable format.

This tool was built for converting from a query of SRA biosamples, though it should be applicable beyond that due to the configurable nature.

Example use case: You want to get the metadata from all the SRA runs of a single bioproject:

```bash
BIOPROJECT_ID=1081646
efetch -db bioproject -id $BIOPROJECT_ID -format xml | elink -db bioproject -target sra > bioproject_query.elink
cat bioproject_query.elink | efetch -mode xml -format xml > biosamples.out.xml
efetch2jsonl -i biosamples.out.xml -o biosamples.jsonl -k . -r EXPERIMENT_PACKAGE
```

Further processing to a clean table can be done trivially with something like Polars.

