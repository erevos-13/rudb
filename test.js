const {DataStore} = require('./rudb-nodejs/index.js');

const store = new DataStore();

try {
  
// Insert multiple documents
const dummyData = [];
for (let i = 0; i < 500_000; i++) { // Generate 10,000 dummy documents
  dummyData.push({
    id: `key${i}`,
    a: Math.floor(Math.random() * 100), // Random number for 'a'
    name: `User ${i}`,
    data: `This is some dummy data for user ${i}. `.repeat(Math.floor(Math.random() * 5) + 1) // Random length string
  });
}
console.time('Insert Time');
store.insert(dummyData);

// Find documents with a specific value

// store.delete({ where: { id: 'key100032' } });
const results = store.findDocuments({
  where: { a: 50 },
  sort: 'name',
  page: 1,
  size: 1,
});
console.timeEnd('Insert Time');
console.log('Results:',results);
console.log('size', store.size())
} catch (error) {
    // Handle errors
    console.error('An error occurred:', error.message);
}