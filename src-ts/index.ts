import wasm from '../pkg/index_bg.wasm';
import pkg from '../pkg';

Promise.all([pkg, wasm]).then(([pkg]) => {
  pkg.start();
});
