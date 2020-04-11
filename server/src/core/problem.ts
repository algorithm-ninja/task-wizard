import { gql } from 'apollo-server-core';
import { AllowNull, Column, DataType, Index, Table, Unique } from 'sequelize-typescript';
import { ApiObject } from '../main/api';
import { createByIdLoader, createSimpleLoader, UuidBaseModel } from '../main/base-model';
import { Resolvers } from '../main/resolver-types';
import { ScoreGradeDomain } from './feedback/score';
import { ProblemMaterialApi } from './material/problem-material';

export const problemSchema = gql`
    type Problem {
        name: ID!
        fileCollection: FileCollection
    }

    input ProblemInput {
        name: ID!
        files: [ID!]!
    }
`;

/** A problem in TuringArena. */
@Table
export class Problem extends UuidBaseModel<Problem> {
    /** Name of the problem, must be a valid identifier. */
    @Unique
    @Column
    @Index
    name!: string;

    /** Files collection that belongs to this problem. */
    @AllowNull(false)
    @Column(DataType.UUIDV4)
    fileCollectionId!: string;
}

export interface ProblemModelRecord {
    Problem: Problem;
}

export class ProblemApi extends ApiObject {
    byId = createByIdLoader(this.ctx, Problem);
    byName = createSimpleLoader((name: string) => this.ctx.table(Problem).findOne({ where: { name } }));
}

export const problemResolvers: Resolvers = {
    Problem: {
        fileCollection: p => ({ uuid: p.fileCollectionId }),
        name: p => p.name,

        title: async (p, {}, ctx) => (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).title,
        statement: async (p, {}, ctx) => (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).statement,
        attachments: async (p, {}, ctx) => (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).attachments,
        awards: async (p, {}, ctx) => (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).awards,
        submissionFields: async (p, {}, ctx) =>
            (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).submissionFields,
        submissionFileTypes: async (p, {}, ctx) =>
            (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).submissionFileTypes,
        submissionFileTypeRules: async (p, {}, ctx) =>
            (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).submissionFileTypeRules,
        submissionListColumns: async (p, {}, ctx) =>
            (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).submissionListColumns,
        evaluationFeedbackColumns: async (p, {}, ctx) =>
            (await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).evaluationFeedbackColumns,
        totalScoreDomain: async (p, {}, ctx) =>
            new ScoreGradeDomain((await ctx.api(ProblemMaterialApi).byProblemId.load(p.id)).scoreRange),
    },
};
