import { useState } from 'react';

const Navbar = () => {
    const [activeDropdown, setActiveDropdown] = useState(null);

    const toggleDropdown = (dropdown) => {
        setActiveDropdown(activeDropdown === dropdown ? null : dropdown);
    };

    // Skeleton callback functions
    const handleSave = () => {
        console.log('Save clicked');
        // Implement save functionality here
    };

    const handleConvexplanoLens = () => {
        console.log('Convexplano lens clicked');
        // Implement convexplano lens example here
    };

    const handlePetzvalLens = () => {
        console.log('Petzval lens clicked');
        // Implement petzval lens example here
    };

    return (
        <nav className="navbar" role="navigation" aria-label="main navigation">
            <div className="navbar-brand">
                <a className="navbar-item" href="/">
                    🍒 Cherry Raytracer
                </a>
                <a aria-expanded="false" aria-label="menu" className="navbar-burger" data-target="navMenu" role="button">
                    <span aria-hidden></span>
                    <span aria-hidden></span>
                    <span aria-hidden></span>
                </a>
            </div>
            <div className="navbar-menu">
                <div className="navbar-start">
                    <div className="navbar-item has-dropdown is-hoverable">
                        <a className="navbar-link" onClick={() => toggleDropdown("file")}>
                            File
                        </a>
                        <div className="navbar-dropdown"><a className="navbar-item" id="file-save" onClick={handleSave}>
                            Save
                        </a></div>
                    </div>

                    <div className="navbar-item has-dropdown is-hoverable">
                        <a className="navbar-link" onClick={() => toggleDropdown("examples")}>
                            Examples
                        </a>
                        <div className="navbar-dropdown">
                            <a className="navbar-item" id="preset-planoconvex" onClick={handleConvexplanoLens}>
                                Convexplano lens
                            </a>
                            <a className="navbar-item" id="preset-petzval" onClick={handlePetzvalLens}>
                                Petzval objective
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    );
};

export default Navbar;
