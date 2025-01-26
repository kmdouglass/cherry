import { useEffect, useState } from 'react';

const MaterialsExplorer = ( {materialsService, isLoadingFullData } ) => {
  const [shelves, setShelves] = useState(new Map());
  const [books, setBooks] = useState(new Map());
  const [pages, setPages] = useState(new Map());
  
  const [selectedShelf, setSelectedShelf] = useState(null);  // Array, 0: key, 1: value
  const [selectedBook, setSelectedBook] = useState(null);    // Array, 0: key, 1: value

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
      .then(pages => setPages(pages || new Map()))
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
    console.log(event.target.value);
  }

  return (
    <div>
      <h1>Materials Explorer</h1>
      <p>Powered by <a href="https://refractiveindex.info/" target="_blank">RefractiveIndex.INFO</a></p>
      <h4 className="title is-4">Shelf</h4>
      <select name="shelves" id="shelves" value={selectedShelf ? selectedShelf[0] : "" } onChange={handleShelfChange}>
        {Array.from(shelves).map(([key, value]) => (
          <option key={key} value={key}>{value}</option>
        ))}
      </select>

      <h4 className="title is-4">Book</h4>
      <select name="books" id="books" value={selectedBook ? selectedBook[0] : ""} onChange={handleBookChange}>
        {Array.from(books).map(([key, value]) => (
          <option key={key} value={key}>{value}</option>
        ))}
      </select>

      <h4 className="title is-4">Page</h4>
      <select name="pages" id="pages" onChange={handlePageChange}>
        {Array.from(pages).map(([key, value]) => (
          <option key={key} value={key}>{value}</option>
        ))}
      </select>
    </div>
  );
};

export default MaterialsExplorer;
