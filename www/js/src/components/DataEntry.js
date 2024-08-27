import { useState } from "react";

import "../css/DataEntry.css";
import SurfacesTable from "./SurfacesTable";

const DataEntry = () => {
  const [activeTab, setActiveTab] = useState('surfaces');

  const renderTabContent = () => {
    switch (activeTab) {
      case 'surfaces':
        return <SurfacesTable />;
      case 'fields':
        return <div>Fields content (to be implemented)</div>;
      case 'aperture':
        return <div>Aperture content (to be implemented)</div>;
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
