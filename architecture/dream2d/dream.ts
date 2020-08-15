
import * as m from 'makerjs';
import * as fs from 'fs';

const PHI = (Math.sqrt(5) + 1) / 2;

const D0 = 44;
const D1 = D0 / Math.pow(PHI, 1);
const D2 = D0 / Math.pow(PHI, 2);
const D3 = D0 / Math.pow(PHI, 3);
const D4 = D0 / Math.pow(PHI, 4);
const D5 = D0 / Math.pow(PHI, 5);
const D6 = D0 / Math.pow(PHI, 6);
const D7 = D0 / Math.pow(PHI, 7);


function p(f) {
  return f * 12;
}

function ft(ft) {
  return ft * 12;
}

function ftin(ft, inch) {
  return ft * 12 + inch;
}

const FULL_WIDTH = ftin(38, 6);
const FULL_HEIGHT = ftin(29, 9);


// function door(width): m.IModel {
//   var res = empty();
  
//   m.$(new m.paths.Line([0, 0], [0, width])).addTo(res, 'leg1');
//   m.$(new m.paths.Line([0, 0], [width, 0])).addTo(res, 'leg2');
//   m.$(new m.paths.Arc([0, 0], width, 0, 90)).addTo(res, 'swing');
  
//   return res;
// }

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

// function walkin() : m.IModel {
//   const res = empty();

//   const width = ftin(4, 10);
//   const height = ftin(9, 6);

//   m.$(new m.models.ConnectTheDots(false, [
//           [ftin(0, 2.25), -ftin(0, 3)],
//           [ftin(0, 2.25), 0],
//           [0, 0],
//           [0, height],
//           [width, height],
//           [width, 0],
//           [width - ftin(0, 7.75), 0],
//           [width - ftin(0, 7.75), -ftin(0, 3)],
//       ])).addTo(res, 'outline');

//   m.$(door(ft(2)))
//     .rotate(180)
//   	.move([width - ftin(0, 7.75), -ftin(0, 3)])
//     .addTo(res, 'door1');
//   m.$(door(ft(2)))
//     .rotate(270)
//   	.move([width - ftin(0, 7.75) - ft(4), -ftin(0, 3)])
//     .addTo(res, 'door2');

//   addSizeCaption(res, "C");
  
//   return res;
// }

// function bedroom() : m.IModel {
//   const top = ftin(13, 9);
//   const bottom = ftin(11, 6);
//   const right = ft(20);
//   const left = ftin(4, 9);

//   const res = empty();

//   m.$(new m.models.ConnectTheDots(false, [
//       [0, right- ftin(1, 5) - ftin(2, 10)],
//       [0, right - left],
//       [top - bottom, right - left],
//       [top - bottom, 0],
//       [top, 0],
//       [top, right],
//       [top - ftin(0, 7.5), right],
//   ])).addTo(res, 'outline1');

//   m.$(new m.models.ConnectTheDots(false, [
//       [top - ftin(0, 7.5) - ft(4), right],
//       [ftin(2, 2.5) + ftin(2, 10), right],
//   ])).addTo(res, 'outline2');

//   m.$(new m.models.ConnectTheDots(false, [
//       [ftin(2, 2.5), right],
//       [0, right],
//       [0, right - ftin(1, 5)],
//   ])).addTo(res, 'outline3');

//   m.$(door(ftin(2, 10)))
//     .rotate(0)
//     .move([0, right - ftin(1, 5) - ftin(2, 10)])
//     .addTo(res, 'entrydoor');
//   addSizeCaption(res, "BED");
//   return res;  
// }

// function ensuite(): m.IModel {
//   const res = empty();

//   m.$(new m.models.ConnectTheDots(false, [
//       [ftin(2, 2.5), 0],
//       [0, 0],
//       [0, ftin(8, 9)],
//       [ftin(8, 5), ftin(8, 9)],
//       [ftin(8, 5), 0],
//       [ftin(2, 2.25) + ftin(2, 10), 0],
//   ])).addTo(res, 'outline');

//   m.$(door(ftin(2, 10)))
//     .rotate(90)
//     .move([ftin(2, 2.5)+ftin(2, 10), 0])
//     .addTo(res, 'door');

//   addSizeCaption(res, "BTH2");
//   return res;  
// }

// function bath(): m.IModel {
//   const res = empty();

//   m.$(new m.models.ConnectTheDots(false, [
//       [ftin(0, 4), 0],
//       [0, 0],
//       [0, ftin(9, 7)],
//       [ftin(5, 5), ftin(9, 7)],
//       [ftin(5, 5), ftin(1, 8)],
//       [ftin(5, 5) - ftin(1, 10), ftin(1, 8)],
//       [ftin(5, 5) - ftin(1, 10), 0],
//       [ftin(0, 4) + ftin(2, 10), 0],
//   ])).addTo(res, 'outline');
  
//   m.$(door(ftin(2, 10)))
//     .move([ftin(0, 4), 0])
//     .addTo(res, 'door');

//   addSizeCaption(res, "BTH1");
//   return res;  
// }

// function toFtIn(measure: number): [number, number] {
//   return [Math.floor(measure / 12), measure % 12];
// }

// function addSizeCaption(model : m.IModel, name: string): void {
//   const ext = m.measure.modelExtents(model);

//   const [wft, win] = toFtIn(ext.width);
//   const [hft, hin] = toFtIn(ext.height);

//   model.caption = {
//     text: `${name}: ${wft}'${win}" x ${hft}'${hin}"`,
//     anchor: new m.paths.Line(ext.center, ext.center),
//   }
// }

// function living(): m.IModel {
//   const res = empty();

//   const w = ftin(14, 11);
//   const h = ftin(19, 8);
//   res.models = {
//     outline1: new m.models.ConnectTheDots(false, [
//         [0, h],
//         [0, 0],
//         [w, 0],
//         [w, ft(15)],
//     ]),
//     outline2: new m.models.ConnectTheDots(false, [
//         [w, ft(19)],
//         [w, h],
//     ])
//   }

//   addSizeCaption(res, 'LVRM');
//   return res;
// //  return m.$(res).addCaption(`14'11" x 19'8"`, ext.center, ext.center).$result;
//   // m.$(door(ftin(2, 10)))
//   //   .move([ftin(0, 4), 0])
//   //   .addTo(res);

// }

function kitchen(): m.IModel {
  return {
    origin: [D4, D2],
    paths:{
    bounds: m.$(new m.paths.Line([0, D6], [0, D3-D6])).layer('outline').$result,
    },
    models: {
      counter: new m.models.ConnectTheDots(false, [
          [0, 0],
          [0, D6],
          [D3 - D6, D6],
          [D3 - D6, D3 - D6],
          [0, D3 - D6],
          [0, D3],
          [D3, D3],
          [D3, 0],
          [0, 0],
      ]),
    },  
  }
  // const w = ftin(14, 11);
  // const h = ftin(8, 9);
  // res.models = {
  //   outline1: new m.models.ConnectTheDots(false, [
  //       [0, 0],
  //       [ft(1), 0],
  //       [ft(1), h],
  //       [ft(10) - ftin(0, 6), h],  // 6 is made up
  //       [ft(10) - ftin(0, 6), h - ftin(2, 2)],
  //       [ft(10), h - ftin(2, 2)],
  //       [ft(10), h],
  //       [w, h],
  //       [w, 0],
  //   ]),

  //   // outline2: new m.models.ConnectTheDots(false, [
  //   //     [w, ft(15)],
  //   //     [w, 0],
  //   //     [0, 0],
  //   // ])
  // }

  // m.$(new m.models.Rectangle(ft(9), ftin(3, 2)))
  //   .move([ft(1), h - ftin(8, 9)])
  //   .addTo(res, 'peninsula');

  // m.$(new m.models.Rectangle(ftin(8, 6), ftin(1, 11)))
  //   .move([ft(1), h - ftin(1, 11)])
  //   .addTo(res, 'cooktop');

  // addSizeCaption(res, "KT")

  // return res;

  // m.$(door(ftin(2, 10)))
  //   .move([ftin(0, 4), 0])
  //   .addTo(res);

}

// function cfr(): m.IModel {
//   const res = {};

//   m.$(new m.models.ConnectTheDots(false, [
//       [0, 0],
//       [0, ftin(12, 7) + ftin(2, 4)],
//       [ftin(3, 10), ftin(12, 7) + ftin(2, 4)],
//       [ftin(3, 10), ftin(12, 7)],
//       [ftin(11, 2), ftin(12, 7)],
//       [ftin(11, 2), 0],
//       [0, 0],
//   ])).addTo(res, 'outline');
  
//   m.$(door(ftin(2, 10)))
//     .rotate(270)
//     .move([ftin(0, 4.5), ftin(12, 7) + ftin(2, 4)])
//     .addTo(res, 'door');
//   addSizeCaption(res, "CFR");
//   return res;  
// }

// function empty() : m.IModel {
//     return {
//         origin: [0, 0],
//         paths: {},
//         models: {},
//         units: 'inch',
//     };
// }

function house() : m.IModel {
  return m.$({
    models: {
      outline: new m.models.Rectangle(D0, D1),
      kitchen: kitchen(),
    },
    units: 'inch'
    // models: {
    //   walkin: m.$(walkin())
    //     .move([FULL_WIDTH - ftin(4, 10), ftin(20, 3)])
    //     .$result,
    //   kitchen: m.$(kitchen())
    //     .move([0, ftin(19, 8)])
    //     .$result,
    //   cfr:  m.$(cfr())
    //     .move([ftin(15, 9) - ftin(0, 4.5) - ftin(0, 4), 0])
    //     .$result,
    //   bedroom:  m.$(bedroom())
    //     .move([FULL_WIDTH - ftin(11, 6)- ftin(2, 2.5), 0])
    //     .$result,
    
    //   ensuite: m.$(ensuite())
    //     .move([FULL_WIDTH - ftin(11, 6)- ftin(2, 2.5), FULL_HEIGHT - ftin(8, 9)])
    //     .$result,

    //   bath: m.$(bath())
    //     .move([ft(10) + ftin(4, 9) + ftin(3, 6) - ftin(0, 4), FULL_HEIGHT - ftin(9, 7)])
    //     .$result,

    //   living: m.$(living())
    //     .$result,
    // }
  })
    .originate()
    .$result;
}

function main() : m.IModel {
  const res = house();

  return res;
}

fs.writeFile("dream.svg", m.exporter.toSVG(main(), {
  layerOptions: {
    "outline": {
      cssStyle: 'stroke-dasharray: 5;'
    }
  }
}), ()=> console.log('done'));


