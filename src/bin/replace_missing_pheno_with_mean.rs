#[macro_use]
extern crate clap;
#[macro_use]
extern crate ndarray;
extern crate saber;

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

use clap::Arg;

use saber::program_flow::OrExit;
use saber::util::{extract_str_arg, get_plink_pheno_data_replace_missing_with_mean};
use saber::util::matrix_util::normalize_vector_inplace;

fn main() {
    let mut app = clap_app!(replace_missing_pheno_with_mean =>
        (version: "0.1")
        (author: "Aaron Zhou")
        (@arg pheno_path: --pheno <PHENO> "required; each row has three fields FID IID pheno")
        (@arg out_path: --out <OUT> "required; output file path")
        (@arg normalize: --normalize "if provided, the output phenotypes will be normalized")
    );
    app = app.arg(
        Arg::with_name("missing_rep")
            .long("miss-coding").short("m").takes_value(true).required(true)
            .allow_hyphen_values(true).help("required; coding of the missing value"));
    let matches = app.get_matches();

    let pheno_path = extract_str_arg(&matches, "pheno_path");
    let out_path = extract_str_arg(&matches, "out_path");
    let missing_rep = extract_str_arg(&matches, "missing_rep");
    let normalize = matches.is_present("normalize");

    println!("phenotype filepath: {}\noutput filepath: {}\nmissing_rep: {}\nnormalize: {}",
             pheno_path, out_path, missing_rep, normalize);

    println!("\n=> generating the phenotype array");
    let (header, fid_vec, iid_vec, mut pheno_arr) =
        get_plink_pheno_data_replace_missing_with_mean(&pheno_path, &missing_rep)
            .unwrap_or_exit(Some("failed to get the phenotype array"));
    println!("pheno_arr.dim: {:?}", pheno_arr.dim());

    if normalize {
        println!("\n=> normalizing the output phenotypes");
        normalize_vector_inplace(&mut pheno_arr, 0);
    }

    println!("\n=> writing the output phenotypes to {}", out_path);
    let f = OpenOptions::new().truncate(true).create(true).write(true).open(out_path.as_str())
                              .unwrap_or_exit(Some(format!("failed to create file {}", out_path)));

    let mut buf = BufWriter::new(f);
    buf.write_fmt(format_args!("{}\n", header))
       .unwrap_or_exit(Some("failed to write to the output file"));

    for (i, val) in pheno_arr.iter().enumerate() {
        buf.write_fmt(format_args!("{} {} {}\n", fid_vec[i], iid_vec[i], val))
           .unwrap_or_exit(Some("failed to write to the output file"));
    }
}