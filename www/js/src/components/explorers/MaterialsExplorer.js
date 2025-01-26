import { useEffect, useState } from 'react';

const MaterialsExplorer = ( {materialsService, isLoadingFullData } ) => {
  const [shelves, setShelves] = useState(new Map());
  const [books, setBooks] = useState(new Map());
  const [pages, setPages] = useState(new Map());
  
  const [selectedShelf, setSelectedShelf] = useState(null);  // Array, 0: key, 1: value
  const [selectedBook, setSelectedBook] = useState(null);    // Array, 0: key, 1: value
  const [selectedPage, setSelectedPage] = useState(null);    // Array, 0: key, 1: value

  const [selectedMaterials, setSelectedMaterials] = useState([]);

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
  
  const handlePageChange = (event) => {
    const pageKey = event.target.value;
    const pageName = pages.get(pageKey);
    setSelectedPage([pageKey, pageName]);
  }

  const handleAddMaterial = () => {
    if (!selectedShelf || !selectedBook || !selectedPage) return;

    const key = `${selectedShelf[0]}:${selectedBook[0]}:${selectedPage[0]}`;
    const name = `${selectedBook[1]} / ${selectedPage[1]}`;  // Don't show the shelf
    
    setSelectedMaterials(prev => [...prev, { key, name }]);
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
            <select multiple className="select is-multiple is-fullwidth mb-4" size="8">
              {selectedMaterials.map(material => (
                <option key={material.key} value={material.key}>
                  {material.name}
                </option>
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
              <button className="button is-danger">
                Remove Material
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default MaterialsExplorer;
