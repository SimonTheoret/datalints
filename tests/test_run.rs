use std::{path::PathBuf, str::FromStr};

use datalints::{Args, inner_main};

#[test]
fn test_inner_main_jsonl() {
    let args = Args {
        path: PathBuf::from_str("tests./test_data/jsonl_test.jsonl").unwrap(),
    };
    inner_main(args).unwrap()
}
