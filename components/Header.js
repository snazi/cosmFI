import PropTypes from 'prop-types';

const Header = (props) => {
  const { children, color } = props;

  return (
    <div data-test="header" className={`bg-${color} relative  flex flex-row justify-around h-24`}>
      <div className="w-full absolute bottom-0 flex flex-row justify-around ">
        {children}
      </div>
    </div>
  );
};

Header.propTypes = {
  color: PropTypes.string,
  children: PropTypes.oneOfType([PropTypes.arrayOf(PropTypes.node), PropTypes.node]),
};

Header.defaultProps = {
  color: 'indigo-navy',
  // children: <div>Fantasy investr</div>
  children: <div />,
};

export default Header;
