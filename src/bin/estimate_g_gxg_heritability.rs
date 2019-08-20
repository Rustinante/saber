use biofile::plink_bed::PlinkBed;
use biofile::plink_bim::{FilelinePartitions, PlinkBim};
use clap::{Arg, clap_app};

use saber::heritability_estimator::estimate_g_gxg_heritability;
use saber::program_flow::OrExit;
use saber::util::{extract_optional_str_arg, extract_str_arg, extract_str_vec_arg, get_bed_bim_fam_path, get_pheno_arr};

fn main() {
    let mut app = clap_app!(estimate_multi_gxg_heritability =>
        (version: "0.1")
        (author: "Aaron Zhou")
        (@arg bfile: --bfile -b <BFILE> "The PLINK prefix for x.bed, x.bim, x.fam is x; required")
        (@arg le_snps_path: --le <LE_SNPS> "Plink file prefix to the SNPs in linkage equilibrium to construct the GxG matrix; required")
        (@arg num_random_vecs: --nrv <NUM_RAND_VECS> "Number of random vectors used to estimate traces; required")
    );
    app = app
        .arg(
            Arg::with_name("pheno_path")
                .long("pheno").short("p").takes_value(true).required(true)
                .multiple(true).number_of_values(1)
                .help("Path to the phenotype file. If there are multiple phenotypes, say PHENO1 and PHENO2, \
                pass the paths one by one as follows: -p PHENO1 -p PHENO2")
        )
        .arg(
            Arg::with_name("trace_outpath")
                .long("save-trace").takes_value(true)
                .help("The output path for saving the trace estimates"))
        .arg(
            Arg::with_name("load_trace")
                .long("load-trace").takes_value(true)
                .help("Use the previously saved trace estimates instead of estimating them from scratch")
        )
        .arg(
            Arg::with_name("partition_file").long("partition").takes_value(true)
        );
    let matches = app.get_matches();

    let bfile = extract_str_arg(&matches, "bfile");
    let le_snps_path = extract_str_arg(&matches, "le_snps_path");
    let trace_outpath = extract_optional_str_arg(&matches, "trace_outpath");
    let load_trace = extract_optional_str_arg(&matches, "load_trace");
    let pheno_path_vec = extract_str_vec_arg(&matches, "pheno_path")
        .unwrap_or_exit(None::<String>);

    let [bed_path, bim_path, fam_path] = get_bed_bim_fam_path(&bfile);
    let [le_snps_bed_path, le_snps_bim_path, le_snps_fam_path] = get_bed_bim_fam_path(&le_snps_path);

    let num_random_vecs = extract_str_arg(&matches, "num_random_vecs")
        .parse::<usize>()
        .unwrap_or_exit(Some("failed to parse num_random_vecs"));
    let g_partition_filepath = extract_optional_str_arg(&matches, "partition_file");

    println!("PLINK bed path: {}\nPLINK bim path: {}\nPLINK fam path: {}", bed_path, bim_path, fam_path);
    println!("LE SNPs bed path: {}\nLE SNPs bim path: {}\nLE SNPs fam path: {}", le_snps_bed_path, le_snps_bim_path, le_snps_fam_path);
    println!("phenotype paths:");
    for (i, path) in pheno_path_vec.iter().enumerate() {
        println!("[{}/{}] {}", i + 1, pheno_path_vec.len(), path);
    }
    println!("num_random_vecs: {}", num_random_vecs);
    println!("G partition filepath: {}", g_partition_filepath.as_ref().unwrap_or(&"".to_string()));

    for (pheno_index, pheno_path) in pheno_path_vec.iter().enumerate() {
        println!("\n=> [{}/{}] estimating the heritability for the phenotype at {}", pheno_index + 1, pheno_path_vec.len(), pheno_path);
        println!("\n=> generating the phenotype array and the genotype matrix");
        let geno_bed = PlinkBed::new(&bed_path, &bim_path, &fam_path)
            .unwrap_or_exit(None::<String>);
        let geno_bim = match &g_partition_filepath {
            Some(partition_filepath) => PlinkBim::new_with_partition_file(&bim_path, partition_filepath)
                .unwrap_or_exit(Some(format!("failed to create PlinkBim from bim file: {} and partition file: {}",
                                             &bim_path, partition_filepath))),
            None => PlinkBim::new(&bim_path)
                .unwrap_or_exit(Some(format!("failed to create PlinkBim from {}", &bim_path))),
        };

        let le_snps_bed = PlinkBed::new(&le_snps_bed_path, &le_snps_bim_path, &le_snps_fam_path)
            .unwrap_or_exit(None::<String>);
        let mut le_snps_bim = PlinkBim::new(&le_snps_bim_path)
            .unwrap_or_exit(Some(format!("failed to create PlinkBim for {}", le_snps_bim_path)));
        let mut le_snps_partition = le_snps_bim.get_chrom_to_fileline_positions()
                                               .unwrap_or_exit(Some(format!("failed to get chrom partitions from {}", le_snps_bim_path)));
        le_snps_partition.remove("23");
        le_snps_bim.set_fileline_partitions(Some(FilelinePartitions::new(le_snps_partition)));
        let pheno_arr = get_pheno_arr(pheno_path)
            .unwrap_or_exit(None::<String>);
        match estimate_g_gxg_heritability(geno_bed, geno_bim, le_snps_bed, le_snps_bim, pheno_arr, num_random_vecs, 20) {
            Err(why) => println!("failed to get heritability estimate for {}: {}", &pheno_path, why),
            Ok(est) => println!("estimate for {}:\n{}", &pheno_path, est)
        };
    }
}