import React, { useState, useEffect } from 'react';
import { renderToString } from 'react-dom/server';

const formatValue = (value) => {
  if (typeof value === 'number') {
    // Check if it's scientific notation
    if (Math.abs(value) < 1e-6) {
      return value.toExponential(4);
    }
    return Number(value.toFixed(4));
  }
  return value;
};


// Reusable table component that can be used in both modal and popup
const SummaryTable = ({ data, wavelengths, sorted_indexes, appModes }) => (
  <div>
    <h3>Paraxial Data</h3>
    <table className="summary-table">
      <thead>
        <tr>
          <th>Parameter</th>
          <th>Value</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>Primary Axial Color</td>
          <td>{formatValue(data["Primary Axial Color"])}</td>
        </tr>
      </tbody>
    </table>

    <h3>Paraxial Data (wavelength-dependent)</h3>
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
            <th scope="col">Wavelength, μm</th>
            {sorted_indexes.map((i) => (
              <th scope="col" key={i}>{wavelengths[i]}</th>
            ))}
          </tr>
        )}
      </thead>
      <tbody>
        {Object.entries(data.subviews).map(([key, values]) => (
          <tr key={key}>
            <td>{key}</td>
            {appModes.refractiveIndex ? (
              <td>{formatValue(values[0])}</td>
            ) : (
              sorted_indexes.map((i) => (
                <td key={i}>{formatValue(values[i])}</td>
              ))
            )}
          </tr>
        ))}
      </tbody>
    </table>
  </div>
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

    // For now we only deal with the Y axis as we don't support toric surfaces
    const newSummary = {
      "Primary Axial Color": description.paraxial_view.primary_axial_color.get("Y") || 0,
      subviews: {}
    };
    
    const newSubviewSummaries = {
        "Aperture Stop (surface index)": {},
        "Effective Focal Length": {},
        "Back Focal Distance": {},
        "Front Focal Distance": {},
        "Paraxial Image Plane Location": {},
        "Paraxial Image Plane Semi-Diameter": {},
        "Entrance Pupil Location": {},
        "Entrance Pupil Semi-Diameter": {},
        "Exit Pupil Location": {},
        "Exit Pupil Semi-Diameter": {},
        "Back Principal Plane Location": {},
        "Front Principal Plane Location": {},
    };

    const subviews = description.paraxial_view.subviews;
    for (const [key, subview] of subviews) {
        let wavelength_index;
        let axis = key.split(':')[1];  // Keys are of the form "wavelength_index:axis"

        // If appModes is set to refractive index, we only extract the first wavelength because the
        // results are the same for all wavelengths.
        if (appModes.refractiveIndex) {
            wavelength_index = 0;
        } else {
            wavelength_index = key.split(':')[0];
        }
        
        // For now we only deal with the Y axis as we don't support toric surfaces
        if (axis !== "Y") continue;

        newSubviewSummaries["Aperture Stop (surface index)"][wavelength_index] = subview.aperture_stop;
        newSubviewSummaries["Effective Focal Length"][wavelength_index] = subview.effective_focal_length;
        newSubviewSummaries["Back Focal Distance"][wavelength_index] = subview.back_focal_distance;
        newSubviewSummaries["Front Focal Distance"][wavelength_index] = subview.front_focal_distance;
        newSubviewSummaries["Paraxial Image Plane Location"][wavelength_index] = subview.paraxial_image_plane.location;
        newSubviewSummaries["Paraxial Image Plane Semi-Diameter"][wavelength_index] = subview.paraxial_image_plane.semi_diameter;
        newSubviewSummaries["Entrance Pupil Location"][wavelength_index] = subview.entrance_pupil.location;
        newSubviewSummaries["Entrance Pupil Semi-Diameter"][wavelength_index] = subview.entrance_pupil.semi_diameter;
        newSubviewSummaries["Exit Pupil Location"][wavelength_index] = subview.exit_pupil.location;
        newSubviewSummaries["Exit Pupil Semi-Diameter"][wavelength_index] = subview.exit_pupil.semi_diameter;
        newSubviewSummaries["Back Principal Plane"][wavelength_index] = subview.back_principal_plane;
        newSubviewSummaries["Front Principal Plane"][wavelength_index] = subview.front_principal_plane;

        // Only extract these values once if appModes is set to refractive index
        if (appModes.refractiveIndex) break;
    }
    newSummary.subviews = newSubviewSummaries;

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

  // Used to sort indexes of the wavelengths array in ascending order by wavelength value
  const sorted_indexes = wavelengths.map((_, i) => i).sort((a, b) => wavelengths[a] - wavelengths[b]);

  // Update popup content when summary changes
  useEffect(() => {
    if (popupWindow && !popupWindow.closed && summary) {
      const tableHTML = renderToString(
        <SummaryTable data={summary} wavelengths={wavelengths} sorted_indexes={sorted_indexes} appModes={appModes} />
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
      <SummaryTable data={summary || {}} wavelengths={wavelengths} sorted_indexes={sorted_indexes} appModes={appModes} />
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
