use flate2::read::GzDecoder;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::{Component, Path, PathBuf};
use std::{fs, io};
use tar::Archive;

const EXCLUDED_SAMPLES: &[&str] = &[
    "1702.07035", // no tex sources
    "1702.07668",
    // "1702.06452", // skeleton.tex is in skip list
];

const SKIP_FILES: &[&str] = &[
    "supplementary.tex",  // datasets/1702/1702.08884.gz
    "atlas_authlist.tex", // datasets/1702/1702.08839.gz
    "Preamble.tex",
    "SuppMat.tex",
    "supp.tex",
    "skeleton.tex",
    "biography.tex",
    "author_information.tex",
    "framed.tex",
    "writeup.tex",
];

const ENTRY_FILES: &[&str] = &[
    "main.tex",        // datasets/1702/1702.08857.gz
    "0_main.tex",      // datasets/1702/1702.08571.gz
    "QPC-Sup-sub.tex", // 1702.08773
    "paper_ACC17_preprint.tex",
    "KirshTLS.tex",
    "Main_arXiv.tex",
    "paper.tex",
    "thesis.tex",
    "arxiv.tex",
    "ieee4double.tex",
    "tightness-dist.tex",
    "lls-connected.tex",
    "flatsArXiv2.tex",
    "Proceedings-420-STAR-on-Cori.tex",
    "Runge_causal_discovery_2018_arxiv.tex",
    "TSE_Joint_PEV_charging_network_and_PV_generation_planning_2.1.tex",
    "ICC2017_Secure_Clustered_Dsitributed_Storage_Against_Eavesdropper_Rev_2017.2.22.tex",
    "w51_review.tex",
    "LumpyPlanet_04_2017.tex",
    "EBEXPaper3.tex",
    "SM0912.tex",
    "setsph37.tex",
    "Wi_Kn_2017_Arxiv.tex",
    "Hoffmann_Antiskyrmion.tex",
    "ijcai17.tex",
    "full-eight-vertex.tex",
    "BigVARV3.tex",
    "BohrSomRevistdAll.tex",
    "WaveParticleExperiment.tex",
];

#[derive(Serialize)]
struct Meta {
    /// Total number of samples
    samples: usize,
    /// Map of sample names to entrypoint files
    #[serde(flatten)]
    entrypoints: HashMap<String, String>,
}

fn sanitize_component(component: &OsStr) -> OsString {
    let fixed = component
        .to_string_lossy()
        .replace(':', "_colon_")
        .replace('\\', "_backslash_")
        .replace('\n', "_newline_")
        .replace('\r', "_carriage_return_")
        .replace('\t', "_tab_");
    OsString::from(fixed)
}

fn find_main_doc(sample: &str, path: &Path) -> Result<Option<String>, Box<dyn Error>> {
    let dir = tempdir::TempDir::new(sample)?;

    let Some(ext) = path.file_name().map(OsStr::to_string_lossy) else {
        return Ok(None);
    };
    if ext.ends_with(".tar.gz") {
        let mut archive = Archive::new(GzDecoder::new(File::open(path)?));

        for entry in archive.entries()? {
            let mut entry = entry?;

            let path = entry.path()?;
            let fixed_path = path
                .components()
                .map(|component| match component {
                    Component::Normal(s) => sanitize_component(s),
                    _ => panic!(),
                })
                .collect::<PathBuf>();
            let out_path = dir.path().join(fixed_path);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(out_path)?;
        }
    } else if ext.ends_with(".gz") {
        let mut archive = GzDecoder::new(File::open(path)?);
        let filename = archive.header().unwrap().filename().unwrap();
        let mut file = File::create(dir.path().join(Path::new(str::from_utf8(filename)?)))?;
        io::copy(&mut archive, &mut file)?;
    } else {
        panic!("Neither TAR nor GZ archive")
    }

    let mut viable = Vec::new();
    let mut count = 0;

    for x in fs::read_dir(&dir)? {
        let x = x?;
        let path = x.path();
        if path.extension() != Some(OsStr::new("tex")) {
            continue;
        }

        count += 1;

        let name = path.file_name().unwrap().to_string_lossy();
        if sample == "arXiv-1702.06452v1" && name == "skeleton.tex" {
            return Ok(Some(name.to_string()));
        } else if SKIP_FILES.contains(&&*name) {
            continue;
        } else if ENTRY_FILES.contains(&&*name) {
            return Ok(Some(name.to_string()));
        }

        use memchr::memmem::find;

        let file = fs::read(&path)?;
        if find(&file, b"\\documentclass").is_some() || find(&file, b"\\bye").is_some() {
            viable.push(name.to_string());
        }
    }

    if viable.is_empty() && count == 1 {
        let x = fs::read_dir(&dir)?.next().unwrap()?;
        return Ok(Some(x.file_name().to_string_lossy().to_string()));
    } else if viable.len() >= 2 {
        return Ok(None);
    }
    Ok(viable.pop())
}

fn prepare(
    sample: Result<fs::DirEntry, io::Error>,
) -> Result<Option<(String, String)>, Box<dyn Error>> {
    let sample = sample?;

    let path = sample.path();

    println!("Preparing {}", path.display());

    let meta = sample.metadata()?;
    let len = meta.len();

    if len < 100 {
        return Ok(None);
    }

    let stem = path.file_stem().unwrap().to_string_lossy();
    if EXCLUDED_SAMPLES.contains(&&*stem) {
        return Ok(None);
    }

    match find_main_doc(&stem, &path)? {
        Some(entry) => Ok(Some((stem.to_string(), entry))),
        None => Ok(None),
    }
}

pub async fn bundle_dataset(dataset: u32) -> Result<(), Box<dyn Error>> {
    let entrypoints = fs::read_dir(format!("datasets/{dataset}"))?
        .map(prepare)
        .filter_map(|val| val.transpose())
        .collect::<Result<HashMap<_, _>, Box<dyn Error>>>()?;

    let output_path = format!("datasets/{dataset}.json");
    let out_file = File::create(&output_path)?;
    serde_json::to_writer(
        out_file,
        &Meta {
            samples: entrypoints.len(),
            entrypoints,
        },
    )?;
    Ok(())
}
