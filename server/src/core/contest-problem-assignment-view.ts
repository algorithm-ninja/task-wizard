import { gql } from 'apollo-server-core';
import { ResolversWithModels } from '../main/resolver-types';
import { ContestAwardAssignment } from './contest-award-assignment';
import { ContestAwardAssignmentView } from './contest-award-assignment-view';
import { ContestProblemAssignment } from './contest-problem-assignment';
import { ContestView } from './contest-view';
import { User } from './user';

export const contestProblemAssignmentViewSchema = gql`
    type ContestProblemAssignmentView {
        assignment: ContestProblemAssignment!
        user: User
        problemSetView: ContestProblemSetView!

        gradingState: GradingState!
        canSubmit: Boolean!
        submissions: [Submission!]!

        awardAssignmentViews: [ContestAwardAssignmentView!]!
    }
`;

export class ContestProblemAssignmentView {
    constructor(readonly assignment: ContestProblemAssignment, readonly user: User | null) {}
}

export const contestProblemAssignmentViewResolvers: ResolversWithModels<{
    ContestProblemAssignmentView: ContestProblemAssignmentView;
}> = {
    ContestProblemAssignmentView: {
        assignment: ({ assignment }) => assignment,
        user: ({ user }) => user,
        problemSetView: async ({ assignment, user }) => new ContestView(await assignment.getContest(), user),
        gradingState: async ({ assignment, user }) => ({
            // TODO
            __typename: 'NumericGradingState',
            domain: {
                __typename: 'NumericGradeDomain',
                max: 100,
                allowPartial: true,
                decimalPrecision: 1,
            },
        }),
        canSubmit: () => true, // TODO
        submissions: () => [], // TODO
        awardAssignmentViews: async ({ assignment, user }) => {
            const problem = await assignment.getProblem();
            const material = await problem.getMaterial();

            return material.awards.map(
                award => new ContestAwardAssignmentView(new ContestAwardAssignment(assignment, award), user),
            );
        },
    },
};
