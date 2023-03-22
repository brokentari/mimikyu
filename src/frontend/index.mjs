import { h, render, Component } from "https://unpkg.com/preact@latest?module";
import htm from "https://unpkg.com/htm?module";
import { useState, useEffect } from 'https://unpkg.com/preact@latest/hooks/dist/hooks.module.js?module';

const html = htm.bind(h);

const rows = Array(16).fill(0).map((_, i) => i + 1);
const cols = Array(32).fill(0).map((_, i) => i + 1);

const modes = ['draw', 'erase'];

let url = new URL('/realtime/draw', window.location.href);
url.protocol = url.protocol.replace("http", "ws");

let ws = new WebSocket(url.href);
ws.onopen = (ev) => {
  let initialPixels = Array(16).fill(0).map((_, _i) => Array(32).fill(0));
  render(html`<${App} pixels=${initialPixels}></${App}>`, document.body);
}

ws.onmessage = (ev) => {
  let json = JSON.parse(ev.data);
  console.log('received data: ', json)
  render(html`<${App} pixels=${json}></${App}>`, document.body);
};

class IroColorPicker extends Component {
  componentDidMount() {
    const { props } = this;

    this.colorPicker = new iro.ColorPicker(this.el, props);

    this.colorPicker.on('color:change', (color) => {
      console.log(color)
      if (props.onColorChange) props.onColorChange(color);
    });
  }

  componentDidUpdate() {
    const { color, ...colorPickerState } = this.props;

    if (color) this.colorPicker.color.set(color);

    this.colorPicker.setState(colorPickerState);
  }

  render() {
    return (
      html`<div ref=${el => this.el = el} />`
    )
  }
}

function GridTable(props) {
  return html`
    <table id=our_table>
      ${rows.map((i) => {
        return html`
          <tr>
            ${cols.map((j) => {
              if (props.pixels[i - 1][j - 1] != 0) {
                let color = props.pixels[i-1][j-1] >>> 0;
                let red = (color >> 16) & 0xFF;
                let green = (color >> 8) & 0xFF;
                let blue = color &0xFF;
                return html`
                  <td id=${i - 1}-${j - 1} class="highlighted" style="background-color: rgb(${red},${green},${blue}" />
                `;
              } else {
                return html`
                  <td id=${i - 1}-${j - 1}/>
              `;
              }
            })}
          </tr>
        `;
      })}
    </table>
    `;
}


function isRightClick(e) {
  if (e.which) {
    return (e.which == 3);
  } else if (e.button) {
    return (e.button == 2);
  }

  return false
}

function App(props) {
  const [currMode, setCurrMode] = useState('draw');
  const [isMouseDown, setIsMouseDown] = useState(false);
  const [currColor, setCurrColor] = useState(null);


  function onModeChange(e) {
    if (e.target.id == 'erase') {
      setCurrColor({rgbString: 'rbg(0, 0, 0)', hexString: '#000000'});
    }

    setCurrMode(e.target.id);
  }

  function onDocumentMouseUp(e) {
    if (isRightClick(e)) {
      return false;
    }

    setIsMouseDown(false);
    document.documentElement.onselectstart = function () { return true;}
  }


  function onCellMouseDown(e) {
    if (isRightClick(e)) {
      return false;
    }

    setIsMouseDown(true);
    onCellClick(e);

    if (typeof e.preventDefault != "undefined") { e.preventDefault() }
    document.documentElement.onselectstart = function () { return false }

  }

  function onCellMouseOver(e) {
    if (isMouseDown) {
      onCellClick(e);
    }
  }

  function onColorChange(c) {
    setCurrColor(c);
  }

  function onCellClick(e) {
    if (currMode == 'draw') {
      if (e.target.classList.contains("highlighted")) {
        return;
      }
      $(e.target).css('background-color', currColor.hexString);
      $(e.target).addClass("highlighted");

    } else if (currMode == 'erase') {
      if (!(e.target.classList.contains("highlighted"))) {
        return;
      }

      $(e.target).css('background-color', currColor.hexString);
      $(e.target).removeClass("highlighted");
    }

    ws.send(JSON.stringify({
      mode: currMode.charAt(0).toUpperCase() + currMode.slice(1),
      x: parseInt(e.target.id.split('-')[0]),
      y: parseInt(e.target.id.split('-')[1]),
      color: currColor.rgbString.replaceAll(' ', '')
    }));

    return false;
  }


  useEffect(() => {
    let cells = document.querySelectorAll('td');
    cells.forEach((cell) => {
      cell.draggable = false;
      cell.addEventListener('mousedown', onCellMouseDown);
      cell.addEventListener('mouseover', onCellMouseOver);
    });
    document.addEventListener("mouseup", onDocumentMouseUp);




    return () => {
      cells.forEach((cell) => {
        cell.removeEventListener('mousedown', onCellMouseDown);
        cell.removeEventListener('mouseover', onCellMouseOver);
      });
      document.removeEventListener("mouseup", onDocumentMouseUp);
    }
  });

  function clearBoard() {
    console.log('clearing board');
    $("#our_table td").removeClass("highlighted");
    ws.send(JSON.stringify({
      mode: 'Clear',
      x: 0,
      y: 0,
      color: ''
    }));
  }

  return html`
    <div id=app>
      <${GridTable} pixels=${props.pixels}></${GridTable}>
      <p id=current-mode>Current mode: ${currMode}</p>
      <div id=mode-buttons>
        ${modes.map((mode) => {
    return html`
            <button id=${mode} onclick=${onModeChange}>${mode}</button>
          `;
  }
  )}
        <button id=clear-button onclick=${clearBoard}>clear</button>
      </div>
      <${IroColorPicker} onColorChange=${onColorChange} />
    </div>
  `
}
