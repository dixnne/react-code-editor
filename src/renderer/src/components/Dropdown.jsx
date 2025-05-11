const Dropdown = ({ icon, items = [] }) => {
  return (
    <div className="dropdown">
      <button className="btn btn-outline-light dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false">
        {icon}
      </button>
      <ul className="dropdown-menu">
        {items.map((item, index) => (
          <li key={index}>
            <button className="dropdown-item" onClick={item.action}>{item.label}</button>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default Dropdown;