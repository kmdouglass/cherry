import { useState } from "react";

import "../css/DataEntry.css";
import ApertureTable from "./ApertureTable";
import FieldsTable from "./FieldsTable";
import SurfacesTable from "./SurfacesTable";

const DataEntry = () => {
  const [activeTab, setActiveTab] = useState('surfaces');
  
  // Application state is stored here.
  const [surfaces, setSurfaces] = useState([
      { type: 'Object', n: 1, thickness: 'Infinity', diam: 25, roc: '' },
      { type: 'Conic', n: 1.515, thickness: 5.3, diam: 25, roc: 25.8 },
      { type: 'Conic', n: 1, thickness: 46.6, diam: 25, roc: 100 },
      { type: 'Image', n: '', thickness: '', diam: 25, roc: '' },
  ]);
  const [fields, setFields] = useState(null);
  const [aperture, setAperture] = useState(null);

  const renderTabContent = () => {
    switch(activeTab) {
      case 'surfaces':
        return <SurfacesTable surfaces={surfaces} setSurfaces={setSurfaces} />;
      case 'fields':
        return <FieldsTable fields={fields} setFields={setFields} />;
      case 'aperture':
        return <ApertureTable aperture={aperture} setAperture={setAperture} />;
      default:
        return null;
    }
  };

  return (
    <div className="data-entry">
      <div className="tabs is-centered">
        <ul>
          <li className={activeTab === 'surfaces' ? 'is-active' : ''}>
            <a onClick={() => setActiveTab('surfaces')}>Surfaces</a>
          </li>
          <li className={activeTab === 'fields' ? 'is-active' : ''}>
            <a onClick={() => setActiveTab('fields')}>Fields</a>
          </li>
          <li className={activeTab === 'aperture' ? 'is-active' : ''}>
            <a onClick={() => setActiveTab('aperture')}>Aperture</a>
          </li>
        </ul>
      </div>
      <div className="tab-content">
        {renderTabContent()}
      </div>
    </div>
  );
};

export default DataEntry;
