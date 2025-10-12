// lattice-v3/core/execution/src/inference/coreml_bridge.m

// CoreML Objective-C Bridge
// Provides C-compatible functions for Rust FFI

#import <CoreML/CoreML.h>
#import <Foundation/Foundation.h>

// Model loading
void* MLModelLoad(const char* path, NSError** error) {
    @autoreleasepool {
        NSString* modelPath = [NSString stringWithUTF8String:path];
        NSURL* modelURL = [NSURL fileURLWithPath:modelPath];

        MLModel* model = [MLModel modelWithContentsOfURL:modelURL error:error];
        if (model) {
            return (__bridge_retained void*)model;
        }
        return NULL;
    }
}

// Model compilation
const char* MLModelCompileModelAtURL(const char* path, NSError** error) {
    @autoreleasepool {
        NSString* modelPath = [NSString stringWithUTF8String:path];
        NSURL* modelURL = [NSURL fileURLWithPath:modelPath];

        NSURL* compiledURL = [MLModel compileModelAtURL:modelURL error:error];
        if (compiledURL) {
            const char* compiledPath = [[compiledURL path] UTF8String];
            return strdup(compiledPath); // Caller must free
        }
        return NULL;
    }
}

// Prediction
void* MLModelPredictFromFeatures(
    void* model,
    void* input,
    void* options,
    NSError** error
) {
    @autoreleasepool {
        MLModel* objcModel = (__bridge MLModel*)model;
        id<MLFeatureProvider> objcInput = (__bridge id<MLFeatureProvider>)input;
        MLPredictionOptions* objcOptions = options ? (__bridge MLPredictionOptions*)options : [[MLPredictionOptions alloc] init];

        id<MLFeatureProvider> output = [objcModel predictionFromFeatures:objcInput
                                                                  options:objcOptions
                                                                    error:error];
        if (output) {
            return (__bridge_retained void*)output;
        }
        return NULL;
    }
}

// Feature provider implementation
@interface SimpleFeatureProvider : NSObject <MLFeatureProvider>
@property (nonatomic, strong) NSMutableDictionary<NSString*, MLFeatureValue*>* features;
@end

@implementation SimpleFeatureProvider

- (instancetype)init {
    self = [super init];
    if (self) {
        _features = [NSMutableDictionary dictionary];
    }
    return self;
}

- (NSSet<NSString*>*)featureNames {
    return [NSSet setWithArray:[_features allKeys]];
}

- (MLFeatureValue*)featureValueForName:(NSString*)featureName {
    return _features[featureName];
}

- (void)setFeatureValue:(MLFeatureValue*)value forName:(NSString*)name {
    _features[name] = value;
}

@end

// Feature provider C API
void* MLFeatureProviderCreate() {
    @autoreleasepool {
        SimpleFeatureProvider* provider = [[SimpleFeatureProvider alloc] init];
        return (__bridge_retained void*)provider;
    }
}

void MLFeatureProviderSetMultiArray(
    void* provider,
    const char* name,
    void* array
) {
    @autoreleasepool {
        SimpleFeatureProvider* objcProvider = (__bridge SimpleFeatureProvider*)provider;
        NSString* featureName = [NSString stringWithUTF8String:name];
        MLMultiArray* objcArray = (__bridge MLMultiArray*)array;

        MLFeatureValue* value = [MLFeatureValue featureValueWithMultiArray:objcArray];
        [objcProvider setFeatureValue:value forName:featureName];
    }
}

void* MLFeatureProviderGetMultiArray(
    void* provider,
    const char* name
) {
    @autoreleasepool {
        id<MLFeatureProvider> objcProvider = (__bridge id<MLFeatureProvider>)provider;
        NSString* featureName = [NSString stringWithUTF8String:name];

        MLFeatureValue* value = [objcProvider featureValueForName:featureName];
        if (value && value.type == MLFeatureTypeMultiArray) {
            return (__bridge_retained void*)value.multiArrayValue;
        }
        return NULL;
    }
}

// MultiArray C API
void* MLMultiArrayCreateWithShape(
    const int* shape,
    int shapeCount,
    int dataType,
    NSError** error
) {
    @autoreleasepool {
        NSMutableArray<NSNumber*>* objcShape = [NSMutableArray arrayWithCapacity:shapeCount];
        for (int i = 0; i < shapeCount; i++) {
            [objcShape addObject:@(shape[i])];
        }

        MLMultiArrayDataType objcDataType;
        switch (dataType) {
            case 65568: // Float32
                objcDataType = MLMultiArrayDataTypeFloat32;
                break;
            case 65600: // Float64
                objcDataType = MLMultiArrayDataTypeDouble;
                break;
            case 131104: // Int32
                objcDataType = MLMultiArrayDataTypeInt32;
                break;
            default:
                objcDataType = MLMultiArrayDataTypeFloat32;
        }

        MLMultiArray* array = [[MLMultiArray alloc] initWithShape:objcShape
                                                          dataType:objcDataType
                                                             error:error];
        if (array) {
            return (__bridge_retained void*)array;
        }
        return NULL;
    }
}

float* MLMultiArrayGetDataPointer(void* array) {
    @autoreleasepool {
        MLMultiArray* objcArray = (__bridge MLMultiArray*)array;
        return (float*)objcArray.dataPointer;
    }
}

const int* MLMultiArrayGetShape(void* array) {
    @autoreleasepool {
        MLMultiArray* objcArray = (__bridge MLMultiArray*)array;
        NSArray<NSNumber*>* shape = objcArray.shape;

        int* shapeArray = (int*)malloc(sizeof(int) * shape.count);
        for (NSUInteger i = 0; i < shape.count; i++) {
            shapeArray[i] = [shape[i] intValue];
        }
        return shapeArray; // Caller must free
    }
}

const int* MLMultiArrayGetStrides(void* array) {
    @autoreleasepool {
        MLMultiArray* objcArray = (__bridge MLMultiArray*)array;
        NSArray<NSNumber*>* strides = objcArray.strides;

        int* stridesArray = (int*)malloc(sizeof(int) * strides.count);
        for (NSUInteger i = 0; i < strides.count; i++) {
            stridesArray[i] = [strides[i] intValue];
        }
        return stridesArray; // Caller must free
    }
}

int MLMultiArrayGetCount(void* array) {
    @autoreleasepool {
        MLMultiArray* objcArray = (__bridge MLMultiArray*)array;
        return (int)objcArray.count;
    }
}

// Memory management
void MLModelRelease(void* model) {
    if (model) {
        CFRelease(model);
    }
}

void MLFeatureProviderRelease(void* provider) {
    if (provider) {
        CFRelease(provider);
    }
}

void MLMultiArrayRelease(void* array) {
    if (array) {
        CFRelease(array);
    }
}

// Error handling
const char* NSErrorGetLocalizedDescription(NSError* error) {
    @autoreleasepool {
        return [error.localizedDescription UTF8String];
    }
}

void NSErrorRelease(NSError* __unused error) {
    // NSError is autoreleased, no explicit release needed
}