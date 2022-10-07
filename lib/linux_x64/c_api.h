// Copyright © 2018 - 2019 H2O.AI Inc. All rights reserved.

#ifndef MOJO_C_API_H_
#define MOJO_C_API_H_

#ifdef __cplusplus
#define EXTERN_C extern "C"
#include <cstdio>
#else
#define EXTERN_C
#include <stdio.h>
#endif

#if defined(_MSC_VER) || defined(_WIN32)
#define MOJO_CAPI_EXPORT EXTERN_C __declspec(dllexport)
#else
#define MOJO_CAPI_EXPORT EXTERN_C
#endif

MOJO_CAPI_EXPORT const char* MOJO_Version();

// mojo data types
typedef enum MOJO_DataType {
  MOJO_UNKNOWN = 1,
  MOJO_FLOAT = 2,
  MOJO_DOUBLE = 3,
  MOJO_INT32 = 4,
  MOJO_INT64 = 5,
  MOJO_STRING = 6,
} MOJO_DataType;

// mojo column type
typedef struct MOJO_Col MOJO_Col;

// create a new mojo column
MOJO_CAPI_EXPORT MOJO_Col* MOJO_NewCol(MOJO_DataType, size_t size, void* data);

// delete the mojo column and free the memory
MOJO_CAPI_EXPORT void MOJO_DeleteCol(MOJO_Col* col);

// the type of data in a mojo column
MOJO_CAPI_EXPORT MOJO_DataType MOJO_Type(MOJO_Col* col);

// extract the data from the mojo column
MOJO_CAPI_EXPORT void* MOJO_Data(MOJO_Col* col);

// mojo frame type
typedef struct MOJO_Frame MOJO_Frame;

// create a new mojo frame
MOJO_CAPI_EXPORT MOJO_Frame* MOJO_NewFrame(MOJO_Col** cols, const char** names,
                                           size_t size);

// delete the mojo frame and free the memory
MOJO_CAPI_EXPORT void MOJO_DeleteFrame(MOJO_Frame* frame);

// number of columns in a mojo frame
MOJO_CAPI_EXPORT size_t MOJO_FrameNcol(MOJO_Frame* frame);

// get a mojo column by name
MOJO_CAPI_EXPORT MOJO_Col* MOJO_GetColByName(MOJO_Frame* frame,
                                             const char* colname);

// mojo model type
typedef struct MOJO_Model MOJO_Model;

// create a new mojo model from mojo file
MOJO_CAPI_EXPORT MOJO_Model* MOJO_NewModel(const char* filename,
                                           const char* tf_lib_prefix);

// delete the mojo model and free the memory
MOJO_CAPI_EXPORT void MOJO_DeleteModel(MOJO_Model* model);

// if the mojo model is valid
MOJO_CAPI_EXPORT int MOJO_IsValid(MOJO_Model* model);

// the timestamp of mojo creation
MOJO_CAPI_EXPORT long MOJO_TimeCreated(MOJO_Model* model);

// number of features in a mojo model
MOJO_CAPI_EXPORT size_t MOJO_FeatureNum(MOJO_Model* model);

// name of features in a mojo model
MOJO_CAPI_EXPORT char** MOJO_FeatureNames(MOJO_Model* model);

// type of features in a mojo model
MOJO_CAPI_EXPORT MOJO_DataType* MOJO_FeatureTypes(MOJO_Model* model);

// number of output in a mojo model
MOJO_CAPI_EXPORT size_t MOJO_OutputNum(MOJO_Model* model);

// name of output in a mojo model
MOJO_CAPI_EXPORT char** MOJO_OutputNames(MOJO_Model* model);

// type of output in a mojo model
MOJO_CAPI_EXPORT MOJO_DataType* MOJO_OutputTypes(MOJO_Model* model);

// missing values from the training dataset
MOJO_CAPI_EXPORT char** MOJO_MissingValues(MOJO_Model* model);

MOJO_CAPI_EXPORT size_t MOJO_MissingValuesNum(MOJO_Model* model);

MOJO_CAPI_EXPORT char* MOJO_UUID(MOJO_Model* model);

// prediction over the mojo frame using the mojo model
MOJO_CAPI_EXPORT void MOJO_Predict(MOJO_Model* model, MOJO_Frame* frame);

#endif  // MOJO_C_API_H_
