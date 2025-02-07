import { useEffect, useState, useSyncExternalStore } from 'react';

const MaterialsExplorer = ( {materialsService, isLoadingFullData } ) => {
  const [shelves, setShelves] = useState(new Map()); // For the dropdown selectors, key: shelfId, value: shelfName
  const [books, setBooks] = useState(new Map());     // For the dropdown selectors, key: bookId, value: bookName
  const [pages, setPages] = useState(new Map());     // For the dropdown selectors; key: pageId, value: pageName
  
  const [selectedShelf, setSelectedShelf] = useState(null);  // Array, 0: key, 1: value
  const [selectedBook, setSelectedBook] = useState(null);    // Array, 0: key, 1: value
  const [selectedPage, setSelectedPage] = useState(null);    // Array, 0: key, 1: value

  const selectedMaterials = useSyncExternalStore(
    // Subscribe callback - must return an unsubscribe function
    (onStoreChange) => {
      const observer = new MutationObserver(onStoreChange);
      
      // Return unsubscribe function
      return () => observer.disconnect();
    },
    // GetSnapshot callback - return current value
    () => materialsService.selectedMaterials
  );

  const [viewingMaterial, setViewingMaterial] = useState(null);           // Materials service to investigate material data; object with material data
  const [selectedListItems, setSelectedListItems] = useState([]);         // Listbox; Array of keys of selected listbox items

 // Fetch shelf names when the component mounts and all data is loaded
 useEffect(() => {
    if (isLoadingFullData) return;

    materialsService.getShelves()
      .then(shelves => {
        setShelves(shelves || new Map());
        // Set initial shelf
        if (!selectedShelf) {
          const firstShelf = Array.from(shelves || [])[0]; // 0: key, 1: value
          setSelectedShelf(firstShelf);
        }
      })
      .catch(error => console.error("Failed to fetch shelf names", error));
  }, [isLoadingFullData, materialsService]);

  // Fetch books when the selected shelf changes
  useEffect(() => {
    if (!selectedShelf) return;

    materialsService.getBooksOnShelf(selectedShelf[1])
      .then(books => {
        setBooks(books || new Map())
        // Set initial book
        if (!selectedBook) {
          const firstBook = Array.from(books || [])[0];
          setSelectedBook(firstBook);
        }
      })
      .catch(error => console.error("Failed to fetch book names", error));
  }, [selectedShelf, materialsService]);

  // Fetch pages when the selected book changes
  useEffect(() => {
    if (!selectedBook) return;
  
    materialsService.getPagesInBookOnShelf(selectedBook[1], selectedShelf[1])
      .then(pages => {
        setPages(pages || new Map())
        // Set initial page
          const firstPage = Array.from(pages || [])[0];
          setSelectedPage(firstPage);

          return new Promise((resolve, reject) => {
            resolve(firstPage);
          })
      })
      .then(firstPage => {
        // Fetch the material data for the first page
        const key = `${selectedShelf[0]}:${selectedBook[0]}:${firstPage[0]}`;
        return materialsService.getMaterialFromDB(key);
      })
      .then(material => {
        // Update the material data viewer
        setViewingMaterial(material);
      })
      .catch(error => console.error("Failed to fetch page names", error));
  }, [selectedBook, selectedShelf, materialsService]);

  const handleShelfChange = (event) => {
    const shelfKey = event.target.value;
    const shelfName = shelves.get(shelfKey);
  
    setSelectedShelf([shelfKey, shelfName]);
    setSelectedBook(null);
  }

  const handleBookChange = (event) => {
    const bookKey = event.target.value;
    const bookName = books.get(bookKey);

    setSelectedBook([bookKey, bookName]);
  }
  
  const handlePageChange = async (event) => {
    const pageKey = event.target.value;
    const pageName = pages.get(pageKey);
    setSelectedPage([pageKey, pageName]);

    // Update the material data viewer
    const key = `${selectedShelf[0]}:${selectedBook[0]}:${pageKey}`;
    const material = await materialsService.getMaterialFromDB(key);
    setViewingMaterial(material);
  }

  const handleAddMaterial = async () => {
    if (!selectedShelf || !selectedBook || !selectedPage) return;

    const key = `${selectedShelf[0]}:${selectedBook[0]}:${selectedPage[0]}`;

    // Check if key is already in materials and return if it is
    if (selectedMaterials.has(key)) return;
    
    await materialsService.addMaterialToSelectedMaterials(key);

    const newMaterials = new Map(materialsService.selectedMaterials);
    setSelectedMaterials(newMaterials);
  }

  const handleRemoveMaterial = () => {
    if (!selectedMaterials) return;

    const newMaterials = new Map(selectedMaterials);
    selectedListItems.forEach(key => newMaterials.delete(key));
    
    setSelectedMaterials(newMaterials);
    materialsService.selectedMaterials = newMaterials;

    setSelectedListItems([]);
  }

  const renderMaterialDetails = () => {
    if (!selectedShelf || !selectedBook || !selectedPage) return;

    if (!viewingMaterial) return;

    const materialData = viewingMaterial.data["0"];
    let wavelengthRange = "Not available";

    if (Object.keys(materialData)[0] === "TabulatedNK") {
      const tabulatedData = materialData.TabulatedNK.data;
      wavelengthRange = `${tabulatedData[0][0]} - ${tabulatedData[tabulatedData.length -1][0]} µm`;
    } else if (Object.keys(materialData)[0] === "TabulatedN") {
      const tabulatedData = materialData.TabulatedN.data;
      wavelengthRange = `${tabulatedData[0][0]} - ${tabulatedData[tabulatedData.length -1][0]} µm`; 
    } else {
      // We don't know what this is ahead of time; could be Formula2, Formula3, etc.
      const dataKey = Object.keys(materialData)[0];
      wavelengthRange = `${materialData[dataKey].wavelength_range[0]} - ${materialData[dataKey].wavelength_range[1]} µm`;
    }

    return (
      <div>
        <h4 className="title is-4">Material Details</h4>
        <p><strong>Name:</strong> {viewingMaterial.page}</p>
        <p><strong>Wavelength range:</strong> {wavelengthRange}</p>
      </div>
    );
  }

  return (
    <div>
      <p className="has-text-centered">Powered by <a href="https://refractiveindex.info/" target="_blank">RefractiveIndex.INFO</a></p>
  
      <div className="columns">
        {/* Left Column */}
        <div className="column is-half">
          <div className="box">
            <h4 className="title is-4">Shelf</h4>
            <div className="select is-fullwidth mb-4">
              <select name="shelves" value={selectedShelf ? selectedShelf[0] : ""} onChange={handleShelfChange}>
                {Array.from(shelves).map(([key, value]) => (
                  <option key={key} value={key}>{value}</option>
                ))}
              </select>
            </div>
  
            <h4 className="title is-4">Book</h4>
            <div className="select is-fullwidth mb-4">
              <select name="books" value={selectedBook ? selectedBook[0] : ""} onChange={handleBookChange}>
                {Array.from(books).map(([key, value]) => (
                  <option key={key} value={key}>{value}</option>
                ))}
              </select>
            </div>
  
            <h4 className="title is-4">Page</h4>
            <div className="select is-fullwidth">
              <select name="pages" value={selectedPage ? selectedPage[0] : ""} onChange={handlePageChange}>
                {Array.from(pages).map(([key, value]) => (
                  <option key={key} value={key}>{value}</option>
                ))}
              </select>
            </div>
          </div>
        </div>
  
        {/* Right Column */}
        <div className="column is-half">
          <div className="box">
            <h4 className="title is-4">Selected Materials</h4>
            <select 
              multiple 
              className="select is-multiple is-fullwidth mb-4" 
              size="8"
              value={selectedListItems}
              onChange={(e) => {
                const selectedOptions = Array.from(e.target.selectedOptions, option => option.value);
                setSelectedListItems(selectedOptions);
              }}
            >
              {Array.from(selectedMaterials).map(([key, material ]) => (
                <option key={key} value={key}>{material.shelf} | {material.book} | {material.page}</option>
              ))}
            </select>
            
            <div className="buttons">
              <button 
                onClick={handleAddMaterial}
                disabled={!(selectedShelf && selectedBook && selectedPage)}
                className="button is-primary"
              >
                Add Material
              </button>
              <button
                onClick={handleRemoveMaterial}
                disabled={selectedListItems.length === 0}
                className="button is-danger"
              >
                Remove Material
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Material Details */}
      <div className="box">
        {renderMaterialDetails()}
      </div>

    </div>
  );
};

export default MaterialsExplorer;
