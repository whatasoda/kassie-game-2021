import wasm from '../pkg/index_bg.wasm';
import pkg from '../pkg';
import '../assets/sample_texture.png';
import '../assets/entities0.png';

Promise.all([pkg, wasm]).then(([pkg]) => {
  console.log(pkg.start());
});
