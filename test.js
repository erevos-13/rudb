const {DataStore} = require('./rudb-nodejs/index.js');

const store = new DataStore();

console.log('Setting item in RUDB...');
store.setItem('key1', "{ foo: 'bar', baz: 42 }");

console.log('Item set successfully!');
console.log('get items', store.getItem('key1'));