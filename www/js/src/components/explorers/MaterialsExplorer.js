import { useEffect, useState } from 'react';

const MaterialsExplorer = ( {materialsService, isLoadingFullData } ) => {
  const [shelves, setShelves] = useState([]);

  useEffect(() => {
    if (isLoadingFullData) {
      return;
    }
    materialsService
      .getShelves()
      .then(shelfNames => {
        setShelves(shelfNames || []);
      })
      .catch((error) => {
        console.error("Failed to fetch shelf names", error);
      });


      return () => {  }
  }, [isLoadingFullData]);
  
  return (
    <div>
      <h1>Materials Explorer</h1>
      <ul>
        {shelves.map((shelf) => (
          <li key={shelf}>{shelf}</li>
        ))}
      </ul>
    </div>
  );
};

export default MaterialsExplorer;
