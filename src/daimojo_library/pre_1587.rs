use std::ffi::c_char;
use crate::daimojo_library::{DaiMojoLibrary, MOJO_DataType, MOJO_Pipeline};

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

pub struct MojoOutputTypesIterator<'a> {
    lib: &'a DaiMojoLibrary,
    pipeline_ptr: *const MOJO_Pipeline,
    index: usize,
    count: usize,
}

impl<'a> MojoOutputTypesIterator<'a> {
    pub(crate) fn new(lib: &'a DaiMojoLibrary, pipeline_ptr: *const MOJO_Pipeline) -> Self {
        let count = unsafe { (*pipeline_ptr).output_count };
        Self {
            lib,
            pipeline_ptr,
            index: 0,
            count,
        }
    }
}

impl<'a> Iterator for MojoOutputTypesIterator<'a> {
    type Item = MOJO_DataType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.index {
            None
        } else {
            let value = unsafe { self.lib.api.MOJO_Output_Type(self.pipeline_ptr, self.index) };
            self.index += 1;
            Some(value)
        }
    }
}

pub struct MojoOutputsIterator<'a> {
    lib: &'a DaiMojoLibrary,
    pipeline_ptr: *const MOJO_Pipeline,
    index: usize,
    count: usize,
}

impl<'a> MojoOutputsIterator<'a> {
    pub(crate) fn new(lib: &'a DaiMojoLibrary, pipeline_ptr: *const MOJO_Pipeline) -> Self {
        let count = unsafe { (*pipeline_ptr).output_count };
        Self {
            lib,
            pipeline_ptr,
            index: 0,
            count,
        }
    }
}

impl<'a> Iterator for MojoOutputsIterator<'a> {
    type Item = (*const c_char, MOJO_DataType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.index {
            None
        } else {
            let value = unsafe {
                (
                    self.lib.api.MOJO_Output_Name(self.pipeline_ptr, self.index),
                    self.lib.api.MOJO_Output_Type(self.pipeline_ptr, self.index),
                )
            };
            self.index += 1;
            Some(value)
        }
    }
}
