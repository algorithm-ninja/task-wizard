import { gql } from 'apollo-server-core';
import { Resolvers } from '../main/resolver-types';
import { MessageApi } from './message';
import { SubmitApi } from './submit';

export const mutationSchema = gql`
    type Mutation {
        init: Boolean!
        submit(submission: SubmissionInput!): Submission!
        logIn(token: String!): AuthResult
        sendMessage(message: MessageInput!): Message!
    }
`;

export interface MutationModelRecord {
    Mutation: {};
}

export const mutationResolvers: Resolvers = {
    Mutation: {
        init: async ({}, {}, ctx) => {
            await ctx.environment.sequelize.sync();

            return true;
        },
        submit: async ({}, { submission }, ctx) => {
            await ctx.authorizeUser(submission.username);

            return ctx.api(SubmitApi).submit(submission);
        },
        logIn: ({}, { token }, ctx) => ctx.environment.authService.logIn(token),
        sendMessage: async ({}, { message }, ctx) => {
            if (typeof message.from === 'string') await ctx.authorizeUser(message.from);
            else await ctx.authorizeAdmin();

            return ctx.api(MessageApi).sendMessage(message);
        },
    },
};
