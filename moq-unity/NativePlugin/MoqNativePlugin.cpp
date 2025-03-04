#include "MoqNativePlugin.h"
#include <cstring>
#include <mutex>
#include <thread>
#include <queue>
#include <condition_variable>
#include <chrono>

// Forward declarations
class MoqClientImpl;
static std::unordered_map<MoqClientHandle, std::unique_ptr<MoqClientImpl>> g_clients;
static std::mutex g_clientsMutex;
static int g_nextClientId = 1;

// Frame structure to hold decoded frame data
struct DecodedFrame {
    int width = 0;
    int height = 0;
    std::vector<uint8_t> data;
    int64_t timestamp = 0;
};

// MoQ client implementation
class MoqClientImpl {
public:
    MoqClientImpl(const char* serverUrl, const char* streamPath, int targetLatencyMs) 
        : serverUrl_(serverUrl), streamPath_(streamPath), targetLatencyMs_(targetLatencyMs),
          connectionStatus_(0), running_(true) {
        
        // Start worker thread
        workerThread_ = std::thread(&MoqClientImpl::workerFunction, this);
    }

    ~MoqClientImpl() {
        // Stop worker thread
        {
            std::lock_guard<std::mutex> lock(mutex_);
            running_ = false;
        }
        condVar_.notify_all();
        
        if (workerThread_.joinable()) {
            workerThread_.join();
        }
    }

    bool update() {
        std::lock_guard<std::mutex> lock(mutex_);
        
        // Check if we have a new frame
        if (!frameQueue_.empty()) {
            // Get the latest frame
            currentFrame_ = std::move(frameQueue_.front());
            frameQueue_.pop();
            hasNewFrame_ = true;
        }
        
        return connectionStatus_ >= 0;
    }

    bool getFrameInfo(int& width, int& height) {
        std::lock_guard<std::mutex> lock(mutex_);
        if (hasNewFrame_) {
            width = currentFrame_.width;
            height = currentFrame_.height;
            return true;
        }
        return false;
    }

    bool getFrameData(uint8_t* buffer, int bufferSize) {
        std::lock_guard<std::mutex> lock(mutex_);
        if (hasNewFrame_ && !currentFrame_.data.empty()) {
            int dataSize = static_cast<int>(currentFrame_.data.size());
            if (bufferSize >= dataSize) {
                memcpy(buffer, currentFrame_.data.data(), dataSize);
                hasNewFrame_ = false;
                return true;
            }
        }
        return false;
    }

    int getConnectionStatus() {
        std::lock_guard<std::mutex> lock(mutex_);
        return connectionStatus_;
    }

private:
    void workerFunction() {
        // This would be the actual implementation using MoQ
        // For now, we'll use a placeholder that generates test frames
        
        // Status: 0 = disconnected, 1 = connecting, 2 = connected, -1 = error
        {
            std::lock_guard<std::mutex> lock(mutex_);
            connectionStatus_ = 1; // Connecting
        }
        
        // Simulate connection delay
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
        
        {
            std::lock_guard<std::mutex> lock(mutex_);
            connectionStatus_ = 2; // Connected
        }
        
        int frameCount = 0;
        
        while (true) {
            {
                std::lock_guard<std::mutex> lock(mutex_);
                if (!running_) break;
                
                // Generate a test frame (moving gradient)
                DecodedFrame frame;
                frame.width = 640;
                frame.height = 480;
                frame.timestamp = frameCount * 16667; // ~60fps in microseconds
                
                // RGBA data
                frame.data.resize(frame.width * frame.height * 4);
                
                // Create a simple animated gradient
                for (int y = 0; y < frame.height; y++) {
                    for (int x = 0; x < frame.width; x++) {
                        int idx = (y * frame.width + x) * 4;
                        
                        // Create animated gradient
                        uint8_t r = static_cast<uint8_t>((x + frameCount) % 255);
                        uint8_t g = static_cast<uint8_t>((y + frameCount * 2) % 255);
                        uint8_t b = static_cast<uint8_t>((x + y + frameCount * 3) % 255);
                        
                        frame.data[idx] = r;     // R
                        frame.data[idx + 1] = g; // G
                        frame.data[idx + 2] = b; // B
                        frame.data[idx + 3] = 255; // A (opaque)
                    }
                }
                
                // Add frame to queue (limit queue size to prevent memory issues)
                if (frameQueue_.size() < 5) {
                    frameQueue_.push(std::move(frame));
                }
            }
            
            frameCount++;
            
            // ~60fps
            std::this_thread::sleep_for(std::chrono::milliseconds(16));
        }
    }

    std::string serverUrl_;
    std::string streamPath_;
    int targetLatencyMs_;
    
    std::thread workerThread_;
    std::mutex mutex_;
    std::condition_variable condVar_;
    bool running_ = false;
    
    int connectionStatus_ = 0;
    
    std::queue<DecodedFrame> frameQueue_;
    DecodedFrame currentFrame_;
    bool hasNewFrame_ = false;
};

// Plugin API implementation
extern "C" {

UNITY_INTERFACE_EXPORT MoqClientHandle UNITY_INTERFACE_API MoqCreateClient(const char* serverUrl, const char* streamPath, int targetLatencyMs) {
    std::lock_guard<std::mutex> lock(g_clientsMutex);
    
    MoqClientHandle handle = g_nextClientId++;
    auto client = std::make_unique<MoqClientImpl>(serverUrl, streamPath, targetLatencyMs);
    g_clients[handle] = std::move(client);
    
    return handle;
}

UNITY_INTERFACE_EXPORT void UNITY_INTERFACE_API MoqDestroyClient(MoqClientHandle client) {
    std::lock_guard<std::mutex> lock(g_clientsMutex);
    
    auto it = g_clients.find(client);
    if (it != g_clients.end()) {
        g_clients.erase(it);
    }
}

UNITY_INTERFACE_EXPORT bool UNITY_INTERFACE_API MoqUpdateClient(MoqClientHandle client) {
    std::lock_guard<std::mutex> lock(g_clientsMutex);
    
    auto it = g_clients.find(client);
    if (it != g_clients.end()) {
        return it->second->update();
    }
    
    return false;
}

UNITY_INTERFACE_EXPORT bool UNITY_INTERFACE_API MoqGetFrameInfo(MoqClientHandle client, int* width, int* height) {
    std::lock_guard<std::mutex> lock(g_clientsMutex);
    
    auto it = g_clients.find(client);
    if (it != g_clients.end()) {
        return it->second->getFrameInfo(*width, *height);
    }
    
    return false;
}

UNITY_INTERFACE_EXPORT bool UNITY_INTERFACE_API MoqGetFrameData(MoqClientHandle client, void* data, int bufferSize) {
    std::lock_guard<std::mutex> lock(g_clientsMutex);
    
    auto it = g_clients.find(client);
    if (it != g_clients.end()) {
        return it->second->getFrameData(static_cast<uint8_t*>(data), bufferSize);
    }
    
    return false;
}

UNITY_INTERFACE_EXPORT int UNITY_INTERFACE_API MoqGetConnectionStatus(MoqClientHandle client) {
    std::lock_guard<std::mutex> lock(g_clientsMutex);
    
    auto it = g_clients.find(client);
    if (it != g_clients.end()) {
        return it->second->getConnectionStatus();
    }
    
    return -1;
}

} // extern "C" 