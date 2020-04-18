import { gql } from 'apollo-server-core';
import { ApiObject } from '../../main/api';
import { Resolvers } from '../../main/resolver-types';
import { typed } from '../../util/types';
import { ContestProblemAssignment } from '../contest-problem-assignment';
import {
    ContestProblemAssignmentUserTackling,
    ContestProblemAssignmentUserTacklingApi,
} from '../contest-problem-assignment-user-tackling';
import { ScoreField } from '../feedback/score';
import { ProblemMaterialApi } from '../material/problem-material';
import { User } from '../user';
import { ContestAwardAssignmentView } from './contest-award-assignment-view';
import { ContestProblemSetView } from './contest-problem-set-view';

export const contestProblemAssignmentViewSchema = gql`
    """
    Refers to a given problem, assigned in a given contest, as seen by a given user or anonymously.
    """
    type ContestProblemAssignmentView {
        "Same problem assigned in same contest"
        assignment: ContestProblemAssignment!
        "Viewing user, or null if anonymous"
        user: User

        "Set of problems in same contest, as seen by same user"
        problemSetView: ContestProblemSetView!

        """
        Same problem assigned in same contest tackled by same user,
        if the user is non-anonymous and allowed to have submissions for this problem in this contest,
        and null otherwise.
        """
        tackling: ContestProblemAssignmentUserTackling

        "Current score seen by the user for this problem in this contest."
        totalScoreField: ScoreField!

        "Awards of this problem assigned in same contest as seen by same user (or anonymously)"
        awardAssignmentViews: [ContestAwardAssignmentView!]!
    }
`;

export interface ContestProblemAssignmentView {
    __typename: 'ContestProblemAssignmentView';
    assignment: ContestProblemAssignment;
    user: User | null;
}

export interface ContestProblemAssignmentViewModelRecord {
    ContestProblemAssignmentView: ContestProblemAssignmentView;
}

export class ContestProblemAssignmentViewApi extends ApiObject {
    getTackling({ assignment, user }: ContestProblemAssignmentView) {
        return user !== null
            ? typed<ContestProblemAssignmentUserTackling>({
                  __typename: 'ContestProblemAssignmentUserTackling',
                  assignment,
                  user,
              })
            : null;
    }

    async getTotalScoreField(view: ContestProblemAssignmentView) {
        const { scoreRange } = await this.ctx.api(ProblemMaterialApi).dataLoader.load(view.assignment.problem);
        const tackling = this.getTackling(view);

        const scoreGrade =
            tackling !== null
                ? await this.ctx.api(ContestProblemAssignmentUserTacklingApi).getScoreGrade(tackling)
                : null;

        return new ScoreField(scoreRange, scoreGrade?.score ?? null);
    }

    async getAwardAssignmentViews({ assignment, user }: ContestProblemAssignmentView) {
        const { awards } = await this.ctx.api(ProblemMaterialApi).dataLoader.load(assignment.problem);

        return awards.map(award =>
            typed<ContestAwardAssignmentView>({
                __typename: 'ContestAwardAssignmentView',
                assignment: { __typename: 'ContestAwardAssignment', award, problemAssignment: assignment },
                user,
            }),
        );
    }

    async getProblemSetView({ user, assignment }: ContestProblemAssignmentView) {
        return typed<ContestProblemSetView>({
            __typename: 'ContestProblemSetView',
            problemSet: {
                __typename: 'ContestProblemSet',
                contest: assignment.problem.contest,
            },
            user,
        });
    }
}

export const contestProblemAssignmentViewResolvers: Resolvers = {
    ContestProblemAssignmentView: {
        assignment: v => v.assignment,
        user: v => v.user,
        problemSetView: async (v, {}, ctx) => ctx.api(ContestProblemAssignmentViewApi).getProblemSetView(v),
        tackling: async (v, {}, ctx) => ctx.api(ContestProblemAssignmentViewApi).getTackling(v),
        totalScoreField: async (v, {}, ctx) => ctx.api(ContestProblemAssignmentViewApi).getTotalScoreField(v),
        awardAssignmentViews: async (v, {}, ctx) => ctx.api(ContestProblemAssignmentViewApi).getAwardAssignmentViews(v),
    },
};
