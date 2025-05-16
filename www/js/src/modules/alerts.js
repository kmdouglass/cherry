/* 
 * Display an alert message on the screen.
 */
const showAlert = (message, color = "#f44336") => {
    // Create alert container if it doesn't exist
    let alertContainer = document.getElementById('alert-container');
    if (!alertContainer) {
        alertContainer = document.createElement('div');
        alertContainer.id = 'alert-container';
        alertContainer.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            z-index: 1000;
            transition: opacity 0.3s ease-in-out;
        `;
        document.body.appendChild(alertContainer);
    }

    // Create new alert element
    const alertElement = document.createElement('div');
    alertElement.style.cssText = `
        background-color: ${color};
        color: white;
        padding: 15px 20px;
        margin-bottom: 10px;
        border-radius: 4px;
        box-shadow: 0 2px 5px rgba(0,0,0,0.2);
        display: flex;
        justify-content: space-between;
        align-items: center;
        min-width: 300px;
    `;

    // Add message
    const textElement = document.createElement('span');
    textElement.textContent = message;
    alertElement.appendChild(textElement);

    // Add close button
    const closeButton = document.createElement('button');
    closeButton.innerHTML = '&times;';
    closeButton.style.cssText = `
        background: none;
        border: none;
        color: white;
        font-size: 20px;
        cursor: pointer;
        padding: 0 5px;
        margin-left: 10px;
    `;
    closeButton.onclick = () => {
        alertElement.style.opacity = '0';
        setTimeout(() => alertElement.remove(), 300);
    };
    alertElement.appendChild(closeButton);

    // Add to container
    alertContainer.appendChild(alertElement);

    // Auto remove after 5 seconds
    setTimeout(() => {
        if (alertElement.parentElement) {
            alertElement.style.opacity = '0';
            setTimeout(() => {
                if (alertElement.parentElement) {
                    alertElement.remove();
                }
            }, 300);
        }
    }, 5000);
};

export default showAlert;
