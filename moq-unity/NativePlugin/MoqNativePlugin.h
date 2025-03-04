#pragma once

#include <unordered_map>
#include <memory>
#include <vector>
#include <string>

// Unity plugin interface
#if defined(_MSC_VER)
    #define UNITY_INTERFACE_API __stdcall
    #define UNITY_INTERFACE_EXPORT __declspec(dllexport)
#else
    #define UNITY_INTERFACE_API
    #define UNITY_INTERFACE_EXPORT __attribute__((visibility("default")))
#endif

// Handle type for MoQ clients
typedef int MoqClientHandle;

#ifdef __cplusplus
extern "C" {
#endif

// Create a new MoQ client
UNITY_INTERFACE_EXPORT MoqClientHandle UNITY_INTERFACE_API MoqCreateClient(const char* serverUrl, const char* streamPath, int targetLatencyMs);

// Destroy a MoQ client
UNITY_INTERFACE_EXPORT void UNITY_INTERFACE_API MoqDestroyClient(MoqClientHandle client);

// Update the client (call once per frame)
UNITY_INTERFACE_EXPORT bool UNITY_INTERFACE_API MoqUpdateClient(MoqClientHandle client);

// Get information about the current frame
UNITY_INTERFACE_EXPORT bool UNITY_INTERFACE_API MoqGetFrameInfo(MoqClientHandle client, int* width, int* height);

// Get the frame data
UNITY_INTERFACE_EXPORT bool UNITY_INTERFACE_API MoqGetFrameData(MoqClientHandle client, void* data, int bufferSize);

// Get connection status
// Returns: 0 = disconnected, 1 = connecting, 2 = connected, negative = error code
UNITY_INTERFACE_EXPORT int UNITY_INTERFACE_API MoqGetConnectionStatus(MoqClientHandle client);

#ifdef __cplusplus
}
#endif 