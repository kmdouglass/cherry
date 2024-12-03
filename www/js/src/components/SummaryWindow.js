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
      <style jsx="true">{`
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
    <style jsx="true">{`
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

const SummaryWindow = ({ description, isOpen, onClose }) => {
  const [summary, setSummary] = useState(null);

  // Update summary whenever description changes
  useEffect(() => {
    if (!description) return;

    const subviews = description.paraxial_view.subviews;

    // Javascript is such shit
    const targetKey = [...subviews.keys()].find(key => 
        Array.isArray(key) && 
        key.length === 2 && 
        key[0] === 0 && 
        key[1] === "Y"
    );

    // Just pull out what we need for now; we can get fancy with processing subviews data later
    const apertureStop = subviews.get(targetKey).aperture_stop;
    const backFocalDistance = subviews.get(targetKey).back_focal_distance;
    const effectiveFocalLength = subviews.get(targetKey).effective_focal_length;
    const entrancePupilLocation = subviews.get(targetKey)["entrance_pupil"]["location"];
    const entrancePupilSemiDiameter = subviews.get(targetKey)["entrance_pupil"]["semi_diameter"];
    const backPrincipalPlane = subviews.get(targetKey).back_principal_plane;

    const newSummary = {
        "Aperture Stop (surface index)": apertureStop,
        "Back Focal Distance": backFocalDistance,
        "Effective Focal Length": effectiveFocalLength,
        "Entrance Pupil Location": entrancePupilLocation,
        "Entrance Pupil Semi-Diameter": entrancePupilSemiDiameter,
        "Back Principal Plane": backPrincipalPlane
    };
    
    setSummary(newSummary);
  }, [description]);
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
          // This is for popup window contents only and not modal contents!

          setPopupWindow(popup);
          popup.document.open();

          // Set up close detection using pagehide event
          // This HAS to go after the popup is opened
          popup.addEventListener('pagehide', () => {
              setPopupWindow(null);
              onClose();
            });
          
          // Create basic HTML structure for the popup
          popup.document.write(`
            <!DOCTYPE html>
            <html>
              <head>
                <title>System Summary</title>
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
                <h2>System Summary</h2>
                <p>Distances are relative to the first surface.</p>
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
    }
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
        System Summary
      </h2>
      <p>Distances are relative to the first surface.</p>
      <SummaryTable data={summary || {}} />
    </Modal>
  );
};

export default SummaryWindow;
