extern crate core;

use std::fmt::{Debug, Display, Formatter};
use std::ops::Sub;
use daimojo::{DaiMojoLibrary, FrameExporter, FrameImporter, MOJO_INT32_NAN, MOJO_Transform_Operations, MOJO_Transform_Operations_Type, RawFrame, RawModel, RawPipeline};
use daimojo::MOJO_DataType::{MOJO_DOUBLE, MOJO_INT32};

const LIB: &str = "libdaimojo.so";

/// This pipeline computes following expression:
/// ```
///         let v1: i32 = input.a + input.a2 + input.a3;
///         let v2: f64 = input.b + input.b2 + input.b3;
/// ```
const SIMPLE_PIPELINE_MOJO: &str = "tests/data/transform_agg_sum_py.mojo";

#[test]
fn simple_metadata() -> anyhow::Result<()> {
    let lib = DaiMojoLibrary::load(LIB)?;

    // model
    let model = RawModel::load(&lib, SIMPLE_PIPELINE_MOJO, "")?;
    assert_eq!("c30815f6-f6cb-475d-9f32-64d4152bce2d", model.uuid().to_str()?);
    assert_eq!("", model.dai_version().to_string_lossy());
    let expected_ops =
        MOJO_Transform_Operations::PREDICT as MOJO_Transform_Operations_Type
        | MOJO_Transform_Operations::CONTRIBS_RAW as MOJO_Transform_Operations_Type;
    assert_eq!(expected_ops, model.supported_ops());
    let feature_types_expected = &[MOJO_INT32, MOJO_INT32, MOJO_INT32, MOJO_DOUBLE, MOJO_DOUBLE, MOJO_DOUBLE, ];
    assert_eq!(feature_types_expected, model.feature_types());
    let feature_names_expected = &["a", "a2", "a3", "b", "b2", "b3"];
    let feature_names: Vec<&str> = model.feature_names_iter()
        .map(|s| s.to_str().unwrap())
        .collect();
    assert_eq!(feature_names_expected, feature_names.as_slice());

    // pipeline
    let pipeline = RawPipeline::new(&model, MOJO_Transform_Operations::PREDICT as MOJO_Transform_Operations_Type)?;
    let output_names_expected: Vec<&str> = pipeline.output_names_iter()
        .map(|s| s.to_str().unwrap())
        .collect();
    assert_eq!(output_names_expected, &["v1", "v2"]);
    Ok(())
}

#[test]
fn simple_predict_memory() -> anyhow::Result<()> {
    let lib = DaiMojoLibrary::load(LIB)?;
    let model = RawModel::load(&lib, SIMPLE_PIPELINE_MOJO, "")?;
    let pipeline = RawPipeline::new(&model, MOJO_Transform_Operations::PREDICT as MOJO_Transform_Operations_Type)?;

    // frame
    let frame = RawFrame::new(&pipeline, 3)?;
    let mut a = frame.input_col(0)?;
    a.unchecked_write_next(0);
    a.unchecked_write_next(432);
    a.unchecked_write_next(724);
    let mut a2 = frame.input_col(1)?;
    a2.unchecked_write_next(0);
    a2.unchecked_write_next(-231);
    a2.unchecked_write_next(234);
    let mut a3 = frame.input_col(2)?;
    a3.unchecked_write_next(0);
    a3.unchecked_write_next(-765);
    a3.unchecked_write_next(0);
    let mut b = frame.input_col(3)?;
    b.unchecked_write_next(0.0);
    b.unchecked_write_next(31.12);
    b.unchecked_write_next(1.1);
    let mut b2 = frame.input_col(3)?;
    b2.unchecked_write_next(0.0);
    b2.unchecked_write_next(-999.25);
    b2.unchecked_write_next(5e-2);
    let mut b3 = frame.input_col(3)?;
    b3.unchecked_write_next(-12.0);
    b3.unchecked_write_next(0.0);
    b3.unchecked_write_next(87e5);

    // transformation
    pipeline.transform(&frame, 0, false)?;

    let mut v1 = frame.output_col(0)?;
    let mut v1b = [0;3];
    v1b[0] = v1.unchecked_read_next();
    v1b[1] = v1.unchecked_read_next();
    v1b[2] = v1.unchecked_read_next();

    assert_eq!([0, 432 - 231 - 765, 724 + 234], v1b);
    //
    Ok(())
}

#[test]
fn simple_predict_csv() -> anyhow::Result<()> {
    const INPUT_CSV: &str = "tests/data/transform_agg_sum_py.input.csv";
    let lib = DaiMojoLibrary::load(LIB)?;
    let model = RawModel::load(&lib, SIMPLE_PIPELINE_MOJO, "")?;
    let pipeline = RawPipeline::new(&model, MOJO_Transform_Operations::PREDICT as MOJO_Transform_Operations_Type)?;

    let frame = RawFrame::new(&pipeline, 3)?;
    let mut rdr = csv::Reader::from_path(INPUT_CSV)?;
    let mut importer = FrameImporter::init(&pipeline, &frame, &mut rdr)?;
    let mut exporter = FrameExporter::init(&pipeline, &frame)?;

    // import batch
    let cnt = importer.import_frame(&mut rdr.records()).unwrap().unwrap();
    assert_eq!(3, cnt);

    // predict
    pipeline.transform(&frame, 0, false)?;

    let mut v1 = frame.output_col(0)?;
    assert_eq!(6, v1.unchecked_read_next());
    assert_eq!(66, v1.unchecked_read_next());
    assert_eq!(/*TODO:7*/MOJO_INT32_NAN-2/*TODO OMG why -2?*/, v1.unchecked_read_next());

    let mut v2 = frame.output_col(1)?;
    assert_eq!(15.0, v2.unchecked_read_next());
    assert_eq!(165.0, v2.unchecked_read_next());
    assert_eq!(18.6, v2.unchecked_read_next());


    // export batch
    exporter.export_frame(cnt)?;


    //TODO compare with expected output
    // const OUTPUT_CSV: &str = "tests/data/transform_agg_sum_py.output.csv";

    Ok(())
}

#[derive(Clone,Copy,Default)]
struct Ops(MOJO_Transform_Operations_Type);

impl From<MOJO_Transform_Operations_Type> for Ops {
    fn from(value: MOJO_Transform_Operations_Type) -> Self {
        Ops(value)
    }
}

impl From<Ops> for MOJO_Transform_Operations_Type {
    fn from(value: Ops) -> Self {
        value.0
    }
}

impl Sub for Ops {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 & !rhs.0)
    }
}

impl Ops {
    pub fn new() -> Self {
        Ops(MOJO_Transform_Operations::PREDICT as MOJO_Transform_Operations_Type)
    }

    pub fn predict(&self) -> bool {
        self.0 & MOJO_Transform_Operations::PREDICT as MOJO_Transform_Operations_Type > 0
    }

    pub fn interval(&self) -> bool {
        self.0 & MOJO_Transform_Operations::INTERVAL as MOJO_Transform_Operations_Type > 0
    }

    pub fn contribs_raw(&self) -> bool {
        self.0 & MOJO_Transform_Operations::CONTRIBS_RAW as MOJO_Transform_Operations_Type > 0
    }

    pub fn contribs_original(&self) -> bool {
        self.0 & MOJO_Transform_Operations::CONTRIBS_ORIGINAL as MOJO_Transform_Operations_Type > 0
    }

    pub fn with_interval(self) -> Self {
        Self(self.0 | MOJO_Transform_Operations::INTERVAL as MOJO_Transform_Operations_Type)
    }

    pub fn with_contribs_raw(self) -> Self {
        Self(self.0 | MOJO_Transform_Operations::CONTRIBS_RAW as MOJO_Transform_Operations_Type)
    }

    pub fn with_contribs_original(self) -> Self {
        Self(self.0 | MOJO_Transform_Operations::CONTRIBS_ORIGINAL as MOJO_Transform_Operations_Type)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl Debug for Ops {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ops{")?;
        Display::fmt(self,f)?;
        f.write_str("}")
    }
}

impl Display for Ops {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const SEP: &str = "|";
        let mut sep = "";
        if self.predict() {
            f.write_str("PREDICT")?;
            sep = SEP;
        }
        if self.interval() {
            f.write_str(sep)?;
            f.write_str("INTERVAL")?;
            sep = SEP;
        }
        if self.contribs_raw() {
            f.write_str(sep)?;
            f.write_str("CONTRIBS_RAW")?;
            sep = SEP;
        }
        if self.contribs_original() {
            f.write_str(sep)?;
            f.write_str("CONTRIBS_ORIGINAL")?;
        }
        Ok(())
    }
}

#[test]
fn test_ops() {
    let all = Ops::new()
        .with_interval()
        .with_contribs_raw()
        .with_contribs_original();

    println!("ops all: {all}");
    println!("ops default: {}", Ops::default());
    println!("ops new: {}", Ops::new());

    let supported = Ops::new().with_contribs_raw();
    println!("ops supported: {supported}");
    let unsupported = all - supported;
    println!("ops unsupported: {unsupported}");
    let requested = Ops::new().with_contribs_original();
    println!("ops requested: {requested}");
    let missing = requested - supported;
    println!("ops unsupported: {missing}");
    assert!(!missing.is_empty());
}
