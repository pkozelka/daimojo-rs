use std::ffi::c_char;
use crate::daimojo_library::{DaiMojoLibrary, MOJO_Pipeline};

pub struct MojoOutputNamesIterator<'a> {
    lib: &'a DaiMojoLibrary,
    pipeline_ptr: *const MOJO_Pipeline,
    index: usize,
    count: usize,
}

impl<'a> MojoOutputNamesIterator<'a> {
    pub(crate) fn new(lib: &'a DaiMojoLibrary, pipeline_ptr: *const MOJO_Pipeline) -> Self
    {
        let count = unsafe { (*pipeline_ptr).output_count };
        Self {
            lib,
            pipeline_ptr,
            index: 0,
            count,
        }
    }
}

impl<'a> Iterator for MojoOutputNamesIterator<'a> {
    type Item = *const c_char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.index {
            None
        } else {
            let value = unsafe { self.lib.api.MOJO_Output_Name(self.pipeline_ptr, self.index) };
            self.index += 1;
            Some(value)
        }
    }
}
