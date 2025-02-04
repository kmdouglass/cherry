import React, { useState, useEffect } from 'react';
import { renderToString } from 'react-dom/server';

// Reusable table component that can be used in both modal and popup
const SummaryTable = ({ data, wavelengths, appModes }) => (
  <table className="summary-table">
    <colgroup>
      <col />
      <col span={appModes.refractiveIndex ? 1 : wavelengths.length} />
    </colgroup>
    <thead>
      <tr>
        <th scope="col">Parameter</th>
        <th colSpan={appModes.refractiveIndex ? 1 : wavelengths.length} scope="colgroup">Value</th>
      </tr>
      {!appModes.refractiveIndex && (
        <tr>
          <th scope="col">Wavelengths</th>
          {wavelengths.map((wavelength, i) => (
            <th scope="col" key={i}>{wavelength}</th>
          ))}
        </tr>
      )}
    </thead>
    <tbody>
      {Object.entries(data).map(([key, value]) => (
        <tr key={key}>
          <td>{key}</td>
          <td>{value}</td>
        </tr>
      ))}
    </tbody>
  </table>
);

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

const SummaryWindow = ({ description, isOpen, wavelengths, appModes, onClose }) => {
  const [summary, setSummary] = useState(null);
  const [popupWindow, setPopupWindow] = useState(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  // Update summary whenever description changes
  useEffect(() => {
    if (!description) return;

    const subviews = description.paraxial_view.subviews;
    const targetKey = [...subviews.keys()].find(key => 
        Array.isArray(key) && 
        key.length === 2 && 
        key[0] === 0 && 
        key[1] === "Y"
    );

    const view = subviews.get(targetKey);
    const newSummary = {
        "Aperture Stop (surface index)": view.aperture_stop,
        "Effective Focal Length": view.effective_focal_length,
        "Back Focal Distance": view.back_focal_distance,
        "Front Focal Distance": view.front_focal_distance,
        "Paraxial Image Plane Location": view.paraxial_image_plane.location,
        "Paraxial Image Plane Semi-Diameter": view.paraxial_image_plane.semi_diameter,
        "Entrance Pupil Location": view.entrance_pupil.location,
        "Entrance Pupil Semi-Diameter": view.entrance_pupil.semi_diameter,
        "Exit Pupil Location": view.exit_pupil.location,
        "Exit Pupil Semi-Diameter": view.exit_pupil.semi_diameter,
        "Back Principal Plane": view.back_principal_plane,
        "Front Principal Plane": view.front_principal_plane
    };
    
    setSummary(newSummary);
  }, [description]);

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
          setIsModalOpen(true);
        } else {
          setPopupWindow(popup);
          popup.document.open();
          
          // Create basic HTML structure with styles
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
                  .summary-table { 
                    border-collapse: collapse; 
                    width: 100%;
                    margin-top: 10px;
                  }
                  .summary-table th, 
                  .summary-table td { 
                    border: 1px solid #ddd; 
                    padding: 8px; 
                    text-align: left; 
                  }
                  .summary-table th { 
                    background-color: #f8f9fa; 
                  }
                  .summary-table tr:hover {
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

          popup.addEventListener('pagehide', () => {
            setPopupWindow(null);
            onClose();
          });
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
      const tableHTML = renderToString(
        <SummaryTable data={summary} wavelengths={wavelengths} appModes={appModes} />
      );
      popupWindow.document.getElementById('root').innerHTML = tableHTML;
    }
  }, [summary, popupWindow, wavelengths, appModes]);

  return (
    <Modal isOpen={isModalOpen} onClose={onClose}>
      <h2 style={{ margin: '0 0 20px 0', paddingRight: '30px' }}>
        System Summary
      </h2>
      <p>Distances are relative to the first surface.</p>
      <SummaryTable data={summary || {}} wavelengths={wavelengths} appModes={appModes} />
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
    </Modal>
  );
};

export default SummaryWindow;
