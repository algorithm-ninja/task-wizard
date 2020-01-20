const path = require('path');

/** @type { import('apollo').ApolloConfig } */
const config = {
  client: {
    includes: ['./src/**/*.ts'],
    excludes: [],
    service: {
      name: 'turingarena-server',
      localSchemaFile: path.resolve(__dirname, '../server/src/generated/graphql.schema.graphql'),
    },
  },
};

module.exports = config;
