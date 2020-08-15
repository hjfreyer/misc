



import * as fs from 'fs';
import * as csg from '@jscad/modeling';
import * as dxfSerializer from '@jscad/json-serializer';

const {translate, center, rotate, mirror, mirrorX, mirrorY, rotateX, rotateY, rotateZ} = csg.transforms;
const {extrudeLinear} = csg.extrusions;
const {cuboid} = csg.primitives;
const {hull} = csg.hulls;
const {union, subtract} = csg.booleans;
const {path2} = csg.geometries;
export interface Polygon {
    vertices : [number, number, number][]
}

export type Material = 'wood'

export interface Part {
    geometry: any
    material: Material
}

export interface House {
    parts: Part[]
}

export interface FrameParams {
    numStuds: number
    spacing: number
    studWidth: number
    studDepth: number
    studHeight: number
}

function makeFrame({} : FrameParams): Part[] {

}

function inch(n) {
    return n / 12;
}

function ft(n) {
    return 12*inch(n);
}
function ftin(f, i) {
    return 12*inch(f) + inch(i);
}


function rightTriangleHyp(hyp : number, angle : number): any {
    return csg.primitives.polygon({points: [
        [0, 0],
        [hyp, 0],
        [hyp * Math.cos(angle)* Math.cos(angle), hyp * Math.sin(angle)* Math.cos(angle)],
    ],
    paths: [[0, 1, 2]] });
}

function rightTriangleLeg(leg : number, angle : number): any {
    return csg.primitives.polygon({points: [
        [0, 0],
        [leg, 0],
        [0, leg / Math.tan(angle)],
    ],
    paths: [[0, 1, 2]] });
}

function box(l : number, w : number, h : number): any {
    const res = csg.primitives.cuboid({size: [l, w, h]});
    return translate([l/2, w/2, h/2], res)
}

function rectangle(l : number, w : number): any {
    const res = csg.primitives.rectangle({size: [l, w]});
    return translate([l/2, w/2], res)
}

type Vec3 = [number, number, number];

interface Params {
    core_size: Vec3,
    pitch: number,
    roof: {
        thickness: number
    }
}

function makeRoof({core_size: [cl, cw, ch], pitch} : Params) {
    const frontLength = (cw + 2*inch(4)) * Math.cos(pitch) - inch(5.5);
    const backLength = (cw + 2*inch(4)) * Math.sin(pitch);
    const dipBelow = (inch(5.5) - inch(3.5) / Math.sin(pitch)) * Math.tan(pitch);
    const frontRafter = translate([inch(4), 0, ch],
                        rotateX(pitch - Math.PI / 2, rotate([Math.PI/2, 0, Math.PI/2], extrudeLinear({height: cl- inch(4) * 2}, csg.primitives.polygon({points: [
        [0, 0],
        [inch(3.5) * Math.sin(pitch) * Math.sin(pitch), inch(3.5) * Math.sin(pitch) * Math.cos(pitch)],
        [inch(5.5), -dipBelow],
        [inch(5.5), frontLength],
        [0, frontLength],
    ]})))));


    const backRafterFull = box(inch(5.5), backLength, cl - inch(4) * 2);
    const subStud = mirrorX(rotateZ(Math.PI - pitch, box(inch(4), ch, cl - inch(4) * 2)));
    const backRafter = translate([cl - inch(4), cw + 2*inch(4), ch], rotateX(pitch - Math.PI/2, rotateZ(Math.PI, rotateY(Math.PI/2, subtract(backRafterFull, subStud)))));

    return union(frontRafter, backRafter);
}

function house(params: Params): any {
    const {core_size: [cl, cw, ch], pitch} = params;
    const frontWall = box(cl, inch(4), ch);
    const leftWall = translate([0, inch(4), 0], box(inch(4), cw, ch));
    const rightWall = translate([cl - inch(8), 0, 0], leftWall);
    const backWall = translate([0, cw + inch(4), 0], frontWall);

    const frontExt = mirror({normal: [0, 1, 0]}, 
        mirror({normal: [1, 0, -1]}, 
       extrudeLinear({height: cl}, rightTriangleLeg(ch, pitch))));

//    const roof = translate([0, 0, ch], rotate([Math.PI/2, 0, Math.PI/2], extrudeLinear({height: cl}, rightTriangleHyp(cw + inch(8), pitch))));
    const roof = makeRoof(params);

    const backExt = 
        translate([0, cw + inch(8), ch],
        rotate([0,Math.PI/2, 0],
        extrudeLinear({height: cl}, rightTriangleHyp(ch, pitch))));

    const ww = ft(6);
    const wh = ft(4);

    const wx = ft(3);
    const wz = ft(2.5)


    const outer = ft(3);
    const frontWindow = translate([wx, 0, wz],
        union(box(ww, inch(4), wh), hull(
        box(ww, 0, wh),
        translate([-outer/2, -12, -outer/2], box(ww + outer, 0, wh+outer))
    )))

    const frontWindows = union(frontWindow, translate([ft(8), 0, 0], frontWindow))


    return union(subtract(union(frontWall, frontExt), frontWindows), backWall, leftWall,
    backExt, roof);
}



function mainmodel(): any {
    return center({axes: [false, false, false]}, house({
        core_size: [ft(44), ft(28), ft(9)], 
        pitch: Math.atan2(12, 3),
        roof: {
            thickness: ft(1),
        }
    }));
}

export default function dreammodel(): Polygon[] {
    return csg.geometries.geom3.toPolygons(mainmodel());
}

// function main () {
//     const out = fs.createWriteStream('./dream.geo');

//     const polys : Polygon[] = csg.geometries.geom3.toPolygons(mainmodel());
//     const rawData = dxfSerializer.serialize({}, mainmodel());
//     fs.writeFile('./dream.json', rawData, (e) => { if (e) throw e});

//     let nextPoint = 1;
//     let nextLine = 1;
//     for (let polyIdx = 0; polyIdx < polys.length; polyIdx++) {
//         const vertices = polys[polyIdx].vertices;

//         for (let vIdx = 0; vIdx < vertices.length; vIdx++) {
//             const [x, y, z] = vertices[vIdx];
//             out.write(`Point(${nextPoint + vIdx}) = {${x}, ${y}, ${z}, 1e-02 };
// `);
//         }
//         out.write('\n');
        
//         for (let vIdx = 0; vIdx < vertices.length - 1; vIdx++) {
//             out.write(`Line(${nextLine + vIdx}) = { ${nextPoint + vIdx}, ${nextPoint + vIdx + 1} };
// `);
//         }
//         out.write('\n');

//         const lines = [...Array(vertices.length - 1)].map((_, vIdx) => nextLine + vIdx);

//         out.write(`Curve Loop(${polyIdx + 1}) = {${lines.join(", ")}};
// `)
//         out.write(`Plane Surface(${polyIdx + 1}) = {${polyIdx + 1}};
// `)
//         out.write('\n');

//         nextPoint += vertices.length;
//         nextLine += vertices.length - 1;
//     }

// }

// main();

// title      : OpenJSCAD.org Logo
// author     : Rene K. Mueller
// license    : MIT License
// revision   : 0.003
// tags       : Logo,Intersection,Sphere,Cube
// file       : logo.jscad


// function $(initial) {
//     let indexed_globals = {};
    
//   // not a fan of this, we have way too many explicit api elements
//   // each top key is a library ie : openscad helpers etc
//   // one level below that is the list of libs
//   // last level is the actual function we want to export to 'local' scope
//   Object.keys(globals).forEach(function (libKey) {
//     const lib = globals[libKey];
//     // console.log(`lib:${libKey}: ${lib}`)
//     Object.keys(lib).forEach(function (libItemKey) {
//       const libItems = lib[libItemKey];
//       // console.log('libItems', libItems)
//       Object.keys(libItems).forEach(function (toExposeKey) {
//         // console.log('toExpose',toExpose )
//        indexed_globals[toExposeKey] = globals[libKey][libItemKey][toExposeKey];
//       });
//     });
//   });
    
//     const handler = {
//         get(target, property) {
//             if (property === 'result') {
//                 return target;
//             }
//             if (property === 'map') {
//                 return (cb) => $(cb(target));
//             }
//             if (property === 'property') {
//                 return (property, cb) => {
//                     const res = translate([0, 0, 0], target);
//                     res.properties[property] = cb(res);
//                     return $(res);
//                 };
//             }
//             return (...args) => {
//                 return $(indexed_globals[property](...args, target));
//             };
//         }
//     };
    
    
//     return new Proxy(initial, handler);
// }


// const WOOD_COLOR = [193 / 255, 154 / 255, 107 / 255];

// function eps_cube(size) {
//     return color([0.95, 0.95, 0.95], cube({size, round: true, radius: 0.1}));

// }

// function eps(width_ft, height_ft, thickness_in) {
//     return eps_cube([width_ft * 12, height_ft * 12, thickness_in]);
// }

// // 
// // const EPS_4_4 = eps_cube([4 * 12, 4 * 12, 4]);

// function brick_row(obj_half, obj_full, grid_dim, ridx, cols) {
//     const res = [];
//     for (let cidx = 0; cidx < cols; cidx++) {
//         if ((ridx + cidx) % 2 === 0) {
//             res.push(translate([cidx * grid_dim, 0, 0], 
//                 cidx === cols - 1 ? obj_half : obj_full
//             ));
//         } else if (cidx === 0) {
//             res.push(obj_half);
//         }
//     }
//     return union(res);
// }

// function brick(obj_half, obj_full, grid_dim, rows, cols) {
// const rowModels = [0, 1]
//     .map(ridx => brick_row(obj_half, obj_full, grid_dim, ridx, cols));
//     const res = [];

//     for (let ridx = 0; ridx < rows; ridx++) {
//             res.push(
//                 translate([0, ridx*grid_dim, 0], 
//                     rowModels[ridx%2]));
//     }
//     return union(res);
// }

// function eps_bricks(rows, cols) {
//     return brick(eps(4, 4, 6), eps(8, 4, 6), 4 * 12, rows, cols);
// }


// function twobyfour(n) {
//     return color(
//         WOOD_COLOR, cube({size: [n, inch(1.5), inch(3.5)], 
//     round: true, 
//     radius: inch(0.05)
        
//     }));
// }

// function wall_frame({n, height, spacing, brace_height_low, brace_gap, offset}) {
//     const width = (n - 1) * spacing;
    
//     const vstud = $(twobyfour(height))
//         .rotate([0, 0, 90])
//         .translate([inch(1.5), 0, 0])
//         .result;
        

//     const half_wide = inch(1.5) / 2;
//     function stud(i) {
//         if (i === 0) {
//             return $(vstud)
//                 .translate([0, inch(1.5), 0])
//                 .result;
//         } else if (i === n - 1) {
//             return $(vstud)
//                 .translate([spacing * i - inch(1.5), inch(1.5), 0])
//                 .result;
//         } else {
//             return $(vstud)
//                 .translate([spacing * i - half_wide, inch(1.5), 0])
//                 .result;
//         }
//     }
    
//     function brace(i) {
//         if (i === 0) {
//             return $(twobyfour(spacing - half_wide * 3))
//                 .translate([inch(1.5) + spacing * i, 
//                     brace_height_low + brace_gap * ((offset + i) % 2), 
//                     0])
//                 .result;
//         }
//         if (i === n-2) {
//             return $(twobyfour(spacing - half_wide * 3))
//                 .translate([spacing * i + half_wide, 
//                     brace_height_low + ((offset + i) % 2), 
//                     0])
//                 .result;
//         }
//         return $(twobyfour(spacing - inch(1.5)))
//             .translate([half_wide + spacing * i, 
//                 brace_height_low + ((offset + i) % 2), 
//                 0])
//             .result;
//     }

//     const plate = twobyfour(width);

//     return $(union([
//         plate,
//         // Top plate isn't modular.
//         //plate.translate([0, height + inch(3), 0]),
//         plate.translate([0, height + inch(1.5), 0]),
//         union([...Array(n)].map((_, i) => stud(i))),
//         union([...Array(n-1)].map((_, i) => brace(i))),
// //        stud(12*8),
//  //       translate([0,0, 12*8], stud(12*8)),
//  //       translate([0,0, 12*8+1.5], stud(12*8)),
//  //       studat(1.5/2),
// //        studat(12*8 - 1.5/2),
//  //       union([...Array(4)].map((_, i) => brace(i+ 1))),

//     ]))
//         .property('bottomLeftBack', 
//             () => new CSG.Connector([0, 0, 0], [0, 1, 0], [0, 0, 1]))
//         .property('bottomRightBack', 
//             () => new CSG.Connector([width, 0, 0], [0, 1, 0], [0, 0, 1]))
//         .result
// }

// function flr() {
//     return union([
//       eps_bricks(40 / 4, 28 / 4),
//   intersection([translate([-2 * 12, -2 * 12, 6], 
//   eps_bricks(40/4 + 1, 28 / 4 + 1)),
//    cube([28 * 12, 40*12, 10*12])
//   ])
//   ]);
// }

// function house_block({core_size, pitch}) {
//     const [core_l, core_w, core_h] = core_size;
    
//     const outer_l = core_h * Math.cos(pitch) + core_l * Math.sin(pitch);
//     const outer_h = core_h * Math.sin(pitch) + core_l * Math.cos(pitch);


//     const core = $(cube({size: core_size}))
//         .color([1, 1, 1])
//         .rotate([0, pitch * 180 / Math.PI - 90, 0])
//         .translate([core_h * Math.cos(pitch), 0, outer_h])
//         .result;
    
//     const inner_box = $(cube({size: [outer_l - ft(2), core_w - ft(2), outer_h * 2 - ft(1)]}))
//         .translate([ft(1), ft(1), 0])
//         .result;

//     const outer = $(core.union(
//             $(cube({size: [outer_l, core_w, outer_h * 2]}))
//             .color([0.5, 0.5, 0.5])
//             .result
//         ))
//         .map(s => s.subtract(inner_box))
//         .property('topLeft', () => new CSG.Connector([0, 0, outer_h * 2], [0, 0, 1], [1, 0, 0]))
//         .translate([-core_h * Math.cos(pitch), 0, -outer_h])
//         .rotate([0, 90 - pitch * 180 / Math.PI, 0])
//         .result;

    
//     // const outer = cascade(outer_box.subtract(inner_box))
//     //     // .color([0.5, 0.5, 0.5])
//     //     // .translate([-core_h * Math.cos(pitch), 0, -outer_h])
//     //     // .rotate([0, 90 - pitch * 180 / Math.PI, 0])
//     //     .result
        
    
//     const ground = $(cube({size: [ft(100), ft(100), -ft(100)]}))
//         .center([true, true, false])
//         .result;
        
//     const main = $(union([outer]))
//         .map(s => s.subtract(ground))
//         .property('roofMount', () => outer.properties.topLeft)
//         .result;
        
        
//     const door = cube({size: [ft(4), ft(1), ft(7)]});
    
//     const big_window = $(cube({size: [ft(4), ft(8), ft(6)]}))
//         .translate([ft(27), ft(2), ft(1.5)])
//         .result;
    
//     const window_bank = union([0, 1, 2].map(
//         i => translate([0, ft(9)*i, 0], big_window)
//     ));
    
//     return main.subtract($(door)
//         .translate([ft(1), 0, 0])
//         .result
//     )
//     .subtract(window_bank);
// }

// function roof({core_size: [core_l, core_w, core_h], pitch, roof: {thickness, overhang}}) {
//     const outer_l = core_h * Math.cos(pitch) + core_l * Math.sin(pitch);
//     const outer_h = core_h * Math.sin(pitch) + core_l * Math.cos(pitch);
    
//     // return cube({size: [
//     //         outer_l + overhang[0] + overhang[2], 
//     //         core_w + overhang[1] + overhang[3], 
//     //         thickness]});
//     return $(cube({size: [outer_l + overhang[0] + overhang[2], 
//         core_w + overhang[1] + overhang[3], thickness]}))
//         .translate([-overhang[0], -overhang[3], 0])
//         .property('topLeft', () => new CSG.Connector([0, 0, 0], [0, 0, 1], [1, 0, 0]))
//         .color([0.5, 0.5, 0.5])
//         .result;
// }

// function framing({core_size}) {
//     const f10 = $(wall_frame({
//         n: 10,
//         height: core_size[2],
//         spacing: inch(16),
//         brace_height_low: ft(5),
//         brace_gap: ft(1),
//         offset: 0,
//     }))
//         .result;
//     const f10b = $(wall_frame({
//         n: 10,
//         height: core_size[2],
//         spacing: inch(16),
//         brace_height_low: ft(5),
//         brace_gap: ft(1),
//         offset: 1,
//     }))
//         .result;
    
//     const origin = new CSG.Connector([0, 0, 0], [0, 0, 1], [1, 0, 0]);
    
//     const f1 = f10.connectTo(f10.properties.bottomLeftBack, 
//         origin,
//         false,
//         0
//     );
//     const f2 = f10b.connectTo(f10b.properties.bottomLeftBack, 
//         f1.properties.bottomRightBack,
//         false,
//         0
//     );
    
//     return union([f1, f2])
//         // .union(f1.connectTo(f1.properties.bottomLeftBack, 
//         //     new CSG.Connector([0, 0, 0], [0, 0, 1], [1, 0, 0]),
//         //     false,
//         //     90
//         // ))
//         // .union(f1.connectTo(f1.properties.bottomLeftBack, 
//         //     new CSG.Connector([0, 0, 0], [0, 0, 1], [1, 0, 0]),
//         //     false,
//         //     90
//         // ));
    
// }

// function house(params) {
//     const block = house_block(params);
//     const rf = $(roof(params))
//         .map(rf => rf.connectTo(rf.properties.topLeft, block.properties.topLeft, false, 0))
//         .result;

//     const resident = $(cube({size: [ft(2), ft(1), ft(6)]}))
//         .color([0, 0, 1])
//         .translate([ft(5), ft(5), 0])
//         .result;
    

//     return center([true, true], union([
//         block, 
//         rf,
//         resident
//     ]));
// }
