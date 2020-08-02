

import * as m from 'makerjs';


import * as fs from 'fs';


function ft(f) {
  return f * 12;
}

function ftin(ft, inch) {
  return ft * 12 + inch;
}

const FULL_WIDTH = ftin(38, 6);
const FULL_HEIGHT = ftin(29, 9);


function door(width): m.IModel {
  var res = empty();
  
  m.$(new m.paths.Line([0, 0], [0, width])).addTo(res, 'leg1');
  m.$(new m.paths.Line([0, 0], [width, 0])).addTo(res, 'leg2');
  m.$(new m.paths.Arc([0, 0], width, 0, 90)).addTo(res, 'swing');
  
  return res;
}

// function outline() {
//   return new m.models.ConnectTheDots(true, [
//           [0,0],
//           [0, ftin(28, 5)],
//           [ft(18), ftin(28, 5)],
//           [ft(18), FULL_HEIGHT],
//           [FULL_WIDTH, FULL_HEIGHT],
//           [FULL_WIDTH, 0]
//       ]);
// }

function walkin() : m.IModel {
  const res = empty();

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
      ])).addTo(res, 'outline');

  m.$(door(ft(2)))
    .rotate(180)
  	.move([width - ftin(0, 7.75), -ftin(0, 3)])
    .addTo(res, 'door1');
  m.$(door(ft(2)))
    .rotate(270)
  	.move([width - ftin(0, 7.75) - ft(4), -ftin(0, 3)])
    .addTo(res, 'door2');
  
  return res;
}

function bedroom() : m.IModel {
  const top = ftin(13, 9);
  const bottom = ftin(11, 6);
  const right = ft(20);
  const left = ftin(4, 9);

  const res = empty();

  m.$(new m.models.ConnectTheDots(false, [
      [0, right- ftin(1, 5) - ftin(2, 10)],
      [0, right - left],
      [top - bottom, right - left],
      [top - bottom, 0],
      [top, 0],
      [top, right],
      [top - ftin(0, 7.5), right],
  ])).addTo(res, 'outline1');

  m.$(new m.models.ConnectTheDots(false, [
      [top - ftin(0, 7.5) - ft(4), right],
      [ftin(2, 2.5) + ftin(2, 10), right],
  ])).addTo(res, 'outline2');

  m.$(new m.models.ConnectTheDots(false, [
      [ftin(2, 2.5), right],
      [0, right],
      [0, right - ftin(1, 5)],
  ])).addTo(res, 'outline3');

  m.$(door(ftin(2, 10)))
    .rotate(0)
    .move([0, right - ftin(1, 5) - ftin(2, 10)])
    .addTo(res, 'entrydoor');

  return res;  
}

function ensuite(): m.IModel {
  const res = empty();

  m.$(new m.models.ConnectTheDots(false, [
      [ftin(2, 2.5), 0],
      [0, 0],
      [0, ftin(8, 9)],
      [ftin(8, 5), ftin(8, 9)],
      [ftin(8, 5), 0],
      [ftin(2, 2.25) + ftin(2, 10), 0],
  ])).addTo(res, 'outline');

  m.$(door(ftin(2, 10)))
    .rotate(90)
    .move([ftin(2, 2.5)+ftin(2, 10), 0])
    .addTo(res, 'door');

  return res;  
}

function bath(): m.IModel {
  const res = empty();

  m.$(new m.models.ConnectTheDots(false, [
      [ftin(0, 4), 0],
      [0, 0],
      [0, ftin(9, 7)],
      [ftin(5, 5), ftin(9, 7)],
      [ftin(5, 5), ftin(1, 8)],
      [ftin(5, 5) - ftin(1, 10), ftin(1, 8)],
      [ftin(5, 5) - ftin(1, 10), 0],
      [ftin(0, 4) + ftin(2, 10), 0],
  ])).addTo(res, 'outline');
  
  m.$(door(ftin(2, 10)))
    .move([ftin(0, 4), 0])
    .addTo(res, 'door');

  return res;  
}

function living(): m.IModel {
  const res = empty();

  const w = ftin(14, 11);
  const h = ftin(28, 5);
  m.$(new m.models.ConnectTheDots(false, [
      [0, 0],
      [0, h - ftin(8, 9)],
      [ft(1), h - ftin(8, 9)],
      [ft(1), h],
      [ft(10) - ftin(0, 6), h],  // 6 is made up
      [ft(10) - ftin(0, 6), h - ftin(2, 2)],
      [ft(10), h - ftin(2, 2)],
      [ft(10), h],
      [w, h],
      [w, ft(19)],
  ])).addTo(res, 'outline1');

  m.$(new m.models.ConnectTheDots(false, [
      [w, ft(15)],
      [w, 0],
      [0, 0],
  ])).addTo(res, 'outline2');

  m.$(new m.models.Rectangle(ft(9), ftin(3, 2)))
    .move([ft(1), h - ftin(8, 9)])
    .addTo(res, 'peninsula');

  m.$(new m.models.Rectangle(ftin(8, 6), ftin(1, 11)))
    .move([ft(1), h - ftin(1, 11)])
    .addTo(res, 'cooktop');

  // m.$(door(ftin(2, 10)))
  //   .move([ftin(0, 4), 0])
  //   .addTo(res);

  return res;  
}

function cfr(): m.IModel {
  const res = {};

  m.$(new m.models.ConnectTheDots(false, [
      [0, 0],
      [0, ftin(12, 7) + ftin(2, 4)],
      [ftin(3, 10), ftin(12, 7) + ftin(2, 4)],
      [ftin(3, 10), ftin(12, 7)],
      [ftin(11, 2), ftin(12, 7)],
      [ftin(11, 2), 0],
      [0, 0],
  ])).addTo(res, 'outline');
  
  m.$(door(ftin(2, 10)))
    .rotate(270)
    .move([ftin(0, 4.5), ftin(12, 7) + ftin(2, 4)])
    .addTo(res, 'door');

  return res;  
}

function empty() : m.IModel {
    return {
        origin: [0, 0],
        paths: {},
        models: {},
        units: 'inch',
    };
}

function apartment() : m.IModel {
  var res = empty();
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
    .addTo(res, "cfr");

  m.$(living())
    .addTo(res, "living");

  return res;
}

function main() : m.IModel {
  return apartment();
}

fs.writeFile("4251hunter.svg", m.exporter.toSVG(main(), {
}), ()=> console.log('done'));


