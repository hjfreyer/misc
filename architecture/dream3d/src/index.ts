import * as THREE from 'three'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';

import model from './dreammodel';

init()

function pointToVector([x, z, y] : [number, number, number]): THREE.Vector3 {
    return new THREE.Vector3(x, y, z);
}

function makeScene() : THREE.Scene {
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0xf0f0f0)

    // roll-over helpers

    const polys = model();

    for (const {vertices} of polys) {
        const [a, b, c] = vertices.slice(0, 3).map(pointToVector);

        const ax1 = b.clone().sub(a).normalize();
        const ax2 = ax1.clone().cross(c.clone().sub(a)).normalize();
        const ax3 = ax1.clone().cross(ax2);

        const planeToShape = new THREE.Matrix4().makeBasis(ax1, ax3, ax2).setPosition(a);
        const shapeToPlane = new THREE.Matrix4().getInverse(planeToShape);

        const faceShape = new THREE.Shape();
        
        faceShape.moveTo(0, 0);
        for (const pt of vertices.slice(1)) {
            const flattened = pointToVector(pt).applyMatrix4(shapeToPlane);
            faceShape.lineTo(flattened.x, flattened.y);
        }
        faceShape.lineTo(0, 0);

        const geometry = new THREE.ShapeGeometry( faceShape ).applyMatrix4(planeToShape);
        var material = new THREE.MeshStandardMaterial();
        var mesh = new THREE.Mesh( geometry, material ) ;
        scene.add( mesh );
    }

    // grid

    var light = new THREE.HemisphereLight( 0xffffbb, 0x080820, 1 );
    scene.add( light );

    const ground = new THREE.PlaneGeometry(100, 100);
    ground.rotateX(-Math.PI / 2);
    const groundMaterial = new THREE.MeshStandardMaterial( {color: 0xffff00} );
    const groundMesh = new THREE.Mesh( ground, groundMaterial );

    scene.add(groundMesh)


    var gridHelper = new THREE.GridHelper(100, 100)
    scene.add(gridHelper)


var axesHelper = new THREE.AxesHelper( 5 );
scene.add( axesHelper );

    return scene;
}

function init() {
    console.log("YOOO")

const scene = makeScene();

    const camera = new THREE.PerspectiveCamera(
        45,
        window.innerWidth / window.innerHeight,
        1,
        10000
    )

    // const camera = new THREE.OrthographicCamera( 
    //     window.innerWidth / - 2, window.innerWidth / 2, 
    //     window.innerHeight / 2, window.innerHeight / - 2, 1, 1000 );
// scene.add( camera );

    const renderer = new THREE.WebGLRenderer({ antialias: true })
    renderer.setPixelRatio(window.devicePixelRatio)
    renderer.setSize(window.innerWidth, window.innerHeight)
    document.body.appendChild(renderer.domElement)

    var controls = new OrbitControls( camera, renderer.domElement );
    camera.position.set(25, 40, 60)
    controls.target = new THREE.Vector3(0, 6, 0)
    controls.update();

    function onWindowResize() {
        camera.aspect = window.innerWidth / window.innerHeight
        camera.updateProjectionMatrix()

        renderer.setSize(window.innerWidth, window.innerHeight)
    }



    window.addEventListener('resize', onWindowResize, false)

    function animate() {

        requestAnimationFrame( animate );

        // required if controls.enableDamping or controls.autoRotate are set to true
        controls.update();

        renderer.render( scene, camera );

    }
    animate();
}

