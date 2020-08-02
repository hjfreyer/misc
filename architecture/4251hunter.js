
var m = require('makerjs');

var fs = require('fs');


function ft(f) {
  return f * 12;
}

function ftin(ft, inch) {
  return ft * 12 + inch;
}

const FULL_WIDTH = ftin(38, 6);
const FULL_HEIGHT = ftin(29, 9);


function door(width) {
  var res = {};
  
  m.$(new m.paths.Line([0, 0], [0, width])).addTo(res);
  m.$(new m.paths.Line([0, 0], [width, 0])).addTo(res);
  m.$(new m.paths.Arc([0, 0], width, 0, 90)).addTo(res);
  
  return res;
}

function outline() {
  return new m.models.ConnectTheDots(true, [
          [0,0],
          [0, ftin(28, 5)],
          [ft(18), ftin(28, 5)],
          [ft(18), FULL_HEIGHT],
          [FULL_WIDTH, FULL_HEIGHT],
          [FULL_WIDTH, 0]
      ]);
}

function walkin() {
  var res = {};

  const width = ftin(4, 10);
  const height = ftin(9, 6);

  m.$(new m.models.ConnectTheDots(false, [
          [ftin(0, 2.25), -ftin(0, 3)],
          [ftin(0, 2.25), 0],
          [0, 0],
          [0, height],
          [width, height],
          [width, 0],
          [width - ftin(0, 7.75), 0],
          [width - ftin(0, 7.75), -ftin(0, 3)],
      ])).addTo(res);

  m.$(door(ft(2)))
    .rotate(180)
  	.move([width - ftin(0, 7.75), -ftin(0, 3)])
    .addTo(res);
  m.$(door(ft(2)))
    .rotate(270)
  	.move([width - ftin(0, 7.75) - ft(4), -ftin(0, 3)])
    .addTo(res);
  
  return res;
}

function bedroom() {
  const top = ftin(13, 9);
  const bottom = ftin(11, 6);
  const right = ft(20);
  const left = ftin(4, 9);

  const res = {};

  m.$(new m.models.ConnectTheDots(false, [
      [0, right- ftin(1, 5) - ftin(2, 10)],
      [0, right - left],
      [top - bottom, right - left],
      [top - bottom, 0],
      [top, 0],
      [top, right],
      [top - ftin(0, 7.5), right],
  ])).addTo(res);

  m.$(new m.models.ConnectTheDots(false, [
      [top - ftin(0, 7.5) - ft(4), right],
      [ftin(2, 2.5) + ftin(2, 10), right],
  ])).addTo(res);

  m.$(new m.models.ConnectTheDots(false, [
      [ftin(2, 2.5), right],
      [0, right],
      [0, right - ftin(1, 5)],
  ])).addTo(res);

  m.$(door(ftin(2, 10)))
    .rotate(270)
    .move([0, right - ftin(1, 5)])
    .addTo(res);

  return res;  
}

function ensuite() {
  const res = {};

  m.$(new m.models.ConnectTheDots(false, [
      [ftin(2, 2.5), 0],
      [0, 0],
      [0, ftin(8, 9)],
      [ftin(8, 5), ftin(8, 9)],
      [ftin(8, 5), 0],
      [ftin(2, 2.25) + ftin(2, 10), 0],
  ])).addTo(res);

  m.$(door(ftin(2, 10)))
    .rotate(90)
    .move([ftin(2, 2.5)+ftin(2, 10), 0])
    .addTo(res);

  return res;  
}

function bath() {
  const res = {};

  m.$(new m.models.ConnectTheDots(false, [
      [ftin(0, 4), 0],
      [0, 0],
      [0, ftin(9, 7)],
      [ftin(5, 5), ftin(9, 7)],
      [ftin(5, 5), ftin(1, 8)],
      [ftin(5, 5) - ftin(1, 10), ftin(1, 8)],
      [ftin(5, 5) - ftin(1, 10), 0],
      [ftin(0, 4) + ftin(2, 10), 0],
  ])).addTo(res);
  
  m.$(door(ftin(2, 10)))
    .move([ftin(0, 4), 0])
    .addTo(res);

  return res;  
}

function living() {
  const res = {};

  m.$(new m.models.ConnectTheDots(false, [
      [0, 0],
      [0, ftin(28, 5)],
      [ftin(14, 11), ftin(28, 5)],
      [ftin(14, 11), 0],
      [0, 0],
  ])).addTo(res);
  
  // m.$(door(ftin(2, 10)))
  //   .move([ftin(0, 4), 0])
  //   .addTo(res);

  return res;  
}

function cfr() {
  const res = {};

  m.$(new m.models.ConnectTheDots(false, [
      [0, 0],
      [0, ftin(12, 7) + ftin(2, 4)],
      [ftin(3, 10), ftin(12, 7) + ftin(2, 4)],
      [ftin(3, 10), ftin(12, 7)],
      [ftin(11, 2), ftin(12, 7)],
      [ftin(11, 2), 0],
      [0, 0],
  ])).addTo(res);
  
  m.$(door(ftin(2, 10)))
    .rotate(270)
    .move([ftin(0, 4.5), ftin(12, 7) + ftin(2, 4)])
    .addTo(res);

  return res;  
}

function apartment() {
  var res = {};
    //makerjs.$(outline()).addTo(this, "outline");
    
  m.$(walkin())
    .move([FULL_WIDTH - ftin(4, 10), ftin(20, 3)])
    .addTo(res, "walkin");

  m.$(bedroom())
    .move([FULL_WIDTH - ftin(11, 6)- ftin(2, 2.5), 0])
    .addTo(res, "bedroom");
  
  m.$(ensuite())
    .move([FULL_WIDTH - ftin(11, 6)- ftin(2, 2.5), FULL_HEIGHT - ftin(8, 9)])
    .addTo(res, "ensuite");

  m.$(bath())
    .move([ft(10) + ftin(4, 9) + ftin(3, 6) - ftin(0, 4), FULL_HEIGHT - ftin(9, 7)])
    .addTo(res, "bath");

  m.$(cfr())
    .move([ftin(15, 9) - ftin(0, 4.5) - ftin(0, 4), 0])
    .addTo(res, "bath");

  m.$(living())
    .addTo(res, "living");

  return res;
}

function main() {
  const res = apartment();
  // const res = cfr();
  res.units = 'inch';
  return res;
}

fs.writeFile("4251hunter.svg", m.exporter.toSVG(main(), {
}), ()=> console.log('done'));


