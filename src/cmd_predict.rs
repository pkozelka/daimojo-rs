use daimojo::FrameExporter;
use daimojo::FrameImporter;
use daimojo::{RawFrame, RawPipeline};

//TODO missing features:
// - various CSV in/out flags
// - support variable column width
// - passing selected input columns to output
// - column name mapping + ignore case
// - headerless csv

/// Minimum size returned by [batch_size_magic].
const MIN_BATCH_SIZE: usize = 1000;

pub fn cmd_predict(pipeline: &RawPipeline, _output: Option<String>, input: Option<String>, batch_size: usize) -> anyhow::Result<u8> {
    let batch_size = batch_size_magic(&input, batch_size)?;

    let frame = RawFrame::new(pipeline, batch_size)?;

    let mut rdr = csv::Reader::from_path(input.unwrap())?;
    let mut importer = FrameImporter::init(&pipeline, &frame, &mut rdr)?;
    let mut exporter = FrameExporter::init(&pipeline, &frame)?;
    // read csv
    let mut rdr_iter = rdr.records();
    while let Some(rows) = importer.import_frame(&mut rdr_iter)? {

        // predict
        pipeline.transform(&frame, rows, false)?;
        log::debug!("-- batch {rows} rows");

        // output csv
        exporter.export_frame(rows)?;
    }
    log::info!("Total rows: {}", exporter.saved_rows);
    //
    Ok(0)
}

/// Heuristics to estimate best batch size possible for given input.
/// The goal is, try to hold everything in memory, but don't request too much of it.
fn batch_size_magic(input: &Option<String>, batch_size: usize) -> std::io::Result<usize> {
    Ok(match (batch_size, &input) {
        (0, None) => MIN_BATCH_SIZE * 10,
        (0, Some(path)) => {
            let input_len = std::fs::metadata(path)?.len();
            let mut batch_size = input_len / 50;
            if batch_size < 1000 {
                batch_size = 1000;
            }
            log::warn!("Batch size was automatically set to {batch_size}");
            batch_size as usize
        }
        (batch_size, _) => batch_size,
    })
}
