// setupComputeTest.js
import { vi } from 'vitest';
import { ComputeService } from '../../src/services/computeService';
import { MSG_OUT_COMPUTE, MSG_OUT_INIT } from '../../src/services/computeContants';

/**
 * Creates a mock worker handle object for testing from the main thread.
 * @returns {Object} - A mock worker object with test helper methods
 */
export function createMockWorker() {
  const mockWorker = {
    onmessage: null,

    postMessage: vi.fn((message) => {
      // Store the last message sent to the mock worker
      mockWorker.lastMessageToWorker = message;
    }),

    terminate: vi.fn(),
    
    // Test helper to simulate messages from the worker
    simulateMessageFromWorker: (data) => {
      if (mockWorker.onmessage) {
        mockWorker.onmessage({ data });
      }
    },

    // Test helper to simulate successful initialization
    simulateInitSuccess: () => {
      mockWorker.simulateMessageFromWorker({msg: MSG_OUT_INIT});
    },

    simulateComputeFinished: (computeService) => {
        const msg = {
            msg: MSG_OUT_COMPUTE,
            requestID: 0,
            rays: [[0,1,2], [3,4,5]],
            errorMessage: null
        }
        computeService.results = msg; // Needed to trigger notifications
        mockWorker.simulateMessageFromWorker(msg);
    },

    simulateComputeError: (computeService, errorMessage) => {
        const msg = {
            msg: MSG_OUT_COMPUTE,
            requestID: 0,
            rays: [],
            errorMessage
        }
        computeService.results = []; // Needed to trigger notifications
        mockWorker.simulateMessageFromWorker(msg);
    }
  };

  return mockWorker;
}

/**
 * Creates a mock subscriber to the compute service.
 * @returns {Object} - A mock subscriber object with test helper methods
 */
export function createMockSubscriber() {
    return {
        onComputeRequested: vi.fn(),
        onComputeFinished: vi.fn(),
        onWorkerBusy: vi.fn(),
        onWorkerIdle: vi.fn()
    }
}

/**
 * Creates a setup for ComputeService tests
 * @returns {Object} - Contains the service, mock worker, and utilities
 */
export function setupComputeServiceTest() {
  // Create a mock worker
  const mockWorker = createMockWorker();
  const mockSubscriber = createMockSubscriber();

  // Create a service with the mock worker
  const computeService = new ComputeService(mockWorker);

  const mockServiceOnMessage = vi.fn();
  
  return {
    computeService,
    mockWorker,
    mockSubscriber,
    mockServiceOnMessage,
    
    // Helper to complete initialization
    async initializeService() {
      // Start the initialization
      const initPromise = computeService.initWorker();
      
      // Simulate worker responding to initialization
      mockWorker.simulateInitSuccess();
      
      // Wait for initialization to complete
      await initPromise;
      computeService.setWorkerMessageHandler(mockServiceOnMessage);
    }
  };
}
