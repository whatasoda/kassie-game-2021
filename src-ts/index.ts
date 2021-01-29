import wasm from '../pkg/index_bg.wasm';
import pkg from '../pkg';
import '../assets/sample_texture.png';
import '../assets/entities0.png';
import '../assets/background.png';

Promise.all([pkg, wasm]).then(([pkg]) => {
  // eslint-disable-next-line no-console
  console.log(pkg.start());
});
