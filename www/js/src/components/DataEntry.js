import { useState } from "react";

import "../css/DataEntry.css";
import ApertureTable from "./ApertureTable";
import FieldsTable from "./FieldsTable";
import SurfacesTable from "./SurfacesTable";

const DataEntry = ({
    surfaces, setSurfaces,
    fields, setFields,
    aperture, setAperture
}) => {
  const [activeTab, setActiveTab] = useState('surfaces');

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
