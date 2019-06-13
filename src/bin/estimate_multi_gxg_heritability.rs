#[macro_use]
extern crate clap;
#[macro_use]
extern crate ndarray;
extern crate saber;

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};

use bio_file_reader::plink_bed::PlinkBed;
use clap::Arg;
use saber::heritability_estimator::{estimate_g_and_multi_gxg_heritability,
                                    estimate_g_and_multi_gxg_heritability_from_saved_traces};
use saber::program_flow::OrExit;
use saber::util::{extract_str_arg, extract_optional_str_arg, get_pheno_arr, write_trace_estimates, load_trace_estimates, get_bed_bim_fam_path};

fn get_le_snp_counts(count_filename: &String) -> Result<Vec<usize>, String> {
    let buf = match OpenOptions::new().read(true).open(count_filename.as_str()) {
        Err(why) => return Err(format!("failed to open {}: {}", count_filename, why)),
        Ok(f) => BufReader::new(f)
    };
    let count_vec: Vec<usize> = buf.lines().map(|l| l.unwrap().parse::<usize>().unwrap()).collect();
    Ok(count_vec)
}

fn main() {
    let mut app = clap_app!(estimate_multi_gxg_heritability =>
        (version: "0.1")
        (author: "Aaron Zhou")
        (@arg bfile: --bfile <BFILE> "required; the PLINK prefix for x.bed, x.bim, x.fam is x")
        (@arg le_snps_path: --le <LE_SNPS> "required; plink file prefix to the SNPs in linkage equilibrium")
        (@arg pheno_filename: --pheno <PHENO> "required; each row is one individual containing one phenotype value")
        (@arg gxg_component_count_filename: --counts -c <COUNTS> "required; a file where each line is the number of LE SNPs for the corresponding GxG component")
        (@arg num_random_vecs: --nrv <NUM_RAND_VECS> "number of random vectors used to estimate traces; required")
    );
    app = app
        .arg(
            Arg::with_name("trace_outpath")
                .long("save-trace").takes_value(true)
                .help("The output path for saving the trace estimates"))
        .arg(
            Arg::with_name("load_trace")
                .long("load-trace").takes_value(true)
                .help("Use the previously saved trace estimates instead of estimating them from scratch")
        );
    let matches = app.get_matches();

    let bfile = extract_str_arg(&matches, "bfile");
    let le_snps_path = extract_str_arg(&matches, "le_snps_path");
    let pheno_filename = extract_str_arg(&matches, "pheno_filename");
    let trace_outpath = extract_optional_str_arg(&matches, "trace_outpath");
    let load_trace = extract_optional_str_arg(&matches, "load_trace");

    let [bed_path, bim_path, fam_path] = get_bed_bim_fam_path(&bfile);
    let [le_snps_bed_path, le_snps_bim_path, le_snps_fam_path] = get_bed_bim_fam_path(&le_snps_path);

    let num_random_vecs = extract_str_arg(&matches, "num_random_vecs")
        .parse::<usize>()
        .unwrap_or_exit(Some("failed to parse num_random_vecs"));
    let gxg_component_count_filename = extract_str_arg(&matches, "gxg_component_count_filename");

    println!("PLINK bed path: {}\nPLINK bim path: {}\nPLINK fam path: {}", bed_path, bim_path, fam_path);
    println!("LE SNPs bed path: {}\nLE SNPs bim path: {}\nLE SNPs fam path: {}",
             le_snps_bed_path, le_snps_bim_path, le_snps_fam_path);
    println!("pheno_filepath: {}\ngxg_component_count_filename: {}\nnum_random_vecs: {}",
             pheno_filename, gxg_component_count_filename, num_random_vecs);

    println!("\n=> generating the phenotype array and the genotype matrix");

    let pheno_arr = get_pheno_arr(&pheno_filename)
        .unwrap_or_exit(None::<String>);

    let mut bed = PlinkBed::new(&bed_path, &bim_path, &fam_path)
        .unwrap_or_exit(None::<String>);
    let geno_arr = bed.get_genotype_matrix()
                      .unwrap_or_exit(Some("failed to get the genotype matrix"));

    let mut le_snps_bed = PlinkBed::new(&le_snps_bed_path, &le_snps_bim_path, &le_snps_fam_path)
        .unwrap_or_exit(None::<String>);
    let le_snps_arr = le_snps_bed.get_genotype_matrix()
                                 .unwrap_or_exit(Some("failed to get the le_snps genotype matrix"));

    let counts = get_le_snp_counts(&gxg_component_count_filename)
        .unwrap_or_exit(Some("failed to get GxG component LE SNP counts"));
    let num_gxg_components = counts.len();

    let mut le_snps_arr_vec = Vec::new();
    let mut acc = 0usize;
    for c in counts.into_iter() {
        println!("GxG component {} expects {} LE SNPs", le_snps_arr_vec.len() + 1, c);
        le_snps_arr_vec.push(le_snps_arr.slice(s![..,acc..acc+c]).to_owned());
        acc += c;
    }

    let heritability_estimate_result = match load_trace {
        None => estimate_g_and_multi_gxg_heritability(geno_arr,
                                                      le_snps_arr_vec,
                                                      pheno_arr,
                                                      num_random_vecs),

        Some(load_path) => {
            let trace_estimates = load_trace_estimates(&load_path)
                .unwrap_or_exit(Some(format!("failed to load the trace estimates from {}", load_path)));
            let expected_dim = (num_gxg_components + 2, num_gxg_components + 2);
            assert_eq!(trace_estimates.dim(), expected_dim,
                       "the loaded trace has dim: {:?} which does not match the expected dimension of {:?}",
                       trace_estimates.dim(), expected_dim);
            estimate_g_and_multi_gxg_heritability_from_saved_traces(geno_arr,
                                                                    le_snps_arr_vec,
                                                                    pheno_arr,
                                                                    num_random_vecs,
                                                                    trace_estimates)
        }
    };

    match heritability_estimate_result {
        Ok((a, _b, h)) => {
            println!("\nvariance estimates on the normalized phenotype:\nG variance: {}", h[0]);
            let mut gxg_var_sum = 0.;
            for i in 1..=num_gxg_components {
                println!("GxG component {} variance: {}", i, h[i]);
                gxg_var_sum += h[i];
            }
            println!("noise variance: {}", h[num_gxg_components + 1]);
            println!("total GxG variance: {}", gxg_var_sum);

            match trace_outpath {
                None => (),
                Some(outpath) => {
                    println!("\n=> writing the trace estimates to {}", outpath);
                    write_trace_estimates(&a, &outpath).unwrap_or_exit(None::<String>);
                }
            };
        }
        Err(why) => {
            eprintln!("{}", why);
            return ();
        }
    };
}
