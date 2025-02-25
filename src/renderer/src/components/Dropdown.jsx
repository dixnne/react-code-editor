const Dropdown = (props) => {
  return (
    <div className="dropdown">
    <button className="btn btn-outline-light dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false">
        {props.icon}
    </button>
    <ul className="dropdown-menu">
        <li><a className="dropdown-item" href="#">Action</a></li>
        <li><a className="dropdown-item" href="#">Another action</a></li>
        <li><a className="dropdown-item" href="#">Something else here</a></li>
    </ul>
    </div>
  )
}

export default Dropdown