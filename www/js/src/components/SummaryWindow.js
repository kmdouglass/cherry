import React, { useState, useEffect } from 'react';

const Modal = ({ isOpen, onClose, children }) => {
  if (!isOpen) return null;

  return (
    <div 
      className="modal-overlay"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="modal-content">
        <button className="modal-close" onClick={onClose}>×</button>
        {children}
      </div>
      <style jsx>{`
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: rgba(0, 0, 0, 0.5);
          display: flex;
          justify-content: center;
          align-items: center;
          z-index: 1000;
        }
        .modal-content {
          background: white;
          padding: 20px;
          border-radius: 8px;
          max-width: 500px;
          width: 90%;
          max-height: 90vh;
          overflow-y: auto;
          position: relative;
        }
        .modal-close {
          position: absolute;
          right: 10px;
          top: 10px;
          border: none;
          background: none;
          font-size: 24px;
          cursor: pointer;
          width: 30px;
          height: 30px;
          display: flex;
          align-items: center;
          justify-content: center;
        }
      `}</style>
    </div>
  );
};

const SummaryTable = ({ data }) => (
  <table className="summary-table">
    <thead>
      <tr>
        <th>Parameter</th>
        <th>Value</th>
      </tr>
    </thead>
    <tbody>
      {Object.entries(data).map(([key, value]) => (
        <tr key={key}>
          <td>{key}</td>
          <td>{value}</td>
        </tr>
      ))}
    </tbody>
    <style jsx>{`
      .summary-table {
        width: 100%;
        border-collapse: collapse;
        margin-top: 10px;
      }
      .summary-table th,
      .summary-table td {
        text-align: left;
        padding: 8px;
        border: 1px solid #ddd;
      }
      .summary-table th {
        background-color: #f8f9fa;
      }
      .summary-table tr:hover {
        background-color: #f5f5f5;
      }
    `}</style>
  </table>
);

const SummaryWindow = ({ summary, isOpen, onClose }) => {
  const [popupWindow, setPopupWindow] = useState(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const isMobile = () => {
    return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent) 
           || window.innerWidth <= 768;
  };

  useEffect(() => {
    if (isOpen) {
      if (isMobile()) {
        setIsModalOpen(true);
      } else {
        const popup = window.open('', 'SummaryWindow', 'width=500,height=400');
        if (popup === null || typeof popup === 'undefined') {
          // Fallback to modal if popup is blocked
          setIsModalOpen(true);
        } else {
          
          setPopupWindow(popup);
          popup.document.open();
          
          // Set up close detection using pagehide event
          // This HAS to be done after the document is open for some reason
          popup.addEventListener('pagehide', () => {
              setPopupWindow(null);
              onClose();
        });

          // Create basic HTML structure for the popup
          popup.document.write(`
            <!DOCTYPE html>
            <html>
              <head>
                <title>System Properties</title>
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <style>
                  body { 
                    font-family: system-ui, -apple-system, sans-serif; 
                    padding: 1rem;
                    margin: 0;
                  }
                  table { 
                    border-collapse: collapse; 
                    width: 100%; 
                  }
                  th, td { 
                    border: 1px solid #ddd; 
                    padding: 8px; 
                    text-align: left; 
                  }
                  th { 
                    background-color: #f8f9fa; 
                  }
                  tr:hover {
                    background-color: #f5f5f5;
                  }
                  h2 {
                    margin: 0 0 20px 0;
                  }
                </style>
              </head>
              <body>
                <h2>Summary Results</h2>
                <div id="root"></div>
              </body>
            </html>
          `);
          popup.document.close();
        }
      }
    }
    
    return () => {
      if (popupWindow && !popupWindow.closed) {
        popupWindow.close();
        setPopupWindow(null);
      }
      setIsModalOpen(false);
    };
  }, [isOpen]);

  // Update popup content when summary changes
  useEffect(() => {
    if (popupWindow && !popupWindow.closed && summary) {
      const content = `
        <table>
          <thead>
            <tr>
              <th>Parameter</th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            ${Object.entries(summary).map(([key, value]) => `
              <tr>
                <td>${key}</td>
                <td>${value}</td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      `;
      
      popupWindow.document.getElementById('root').innerHTML = content;
    }
  }, [summary, popupWindow]);

  return (
    <Modal isOpen={isModalOpen} onClose={onClose}>
      <h2 style={{ margin: '0 0 20px 0', paddingRight: '30px' }}>
        System Properties
      </h2>
      <SummaryTable data={summary || {}} />
    </Modal>
  );
};

export default SummaryWindow;