import { gql } from 'apollo-server-core';
import { ApiObject } from '../main/api';
import { ApiContext } from '../main/api-context';
import { ContestApi } from './contest';
import { ContestAwardAssignment } from './contest-award-assignment';
import {
    ContestAwardAssignmentUserTackling,
    ContestAwardAssignmentUserTacklingApi,
} from './contest-award-assignment-user-tackling';
import { ContestProblemAssignment } from './contest-problem-assignment';
import { ScoreGrade } from './feedback/score';
import { ProblemMaterialApi } from './material/problem-material';
import { SubmissionApi } from './submission';
import { User } from './user';
import { ContestProblemAssignmentView } from './view/contest-problem-assignment-view';

export const contestProblemAssignmentUserTacklingSchema = gql`
    """
    Refers to a given problem, assigned in a given contest, tackled by a given user.
    Tackling means having a collection of submissions, and possibly submit a new one.
    This is separate from ContestProblemAssignmentView since a ContestProblemAssignmentUserTackling
    is available only for non-anonymous users who are allowed to have submissions (e.g., only after the contest is started).
    """
    type ContestProblemAssignmentUserTackling {
        "Same problem assigned in same contest as seen by same user"
        assignmentView: ContestProblemAssignmentView!

        "User tackling the problem"
        user: User!

        "List of submissions for this problem in this contest from this user"
        submissions: [Submission!]!

        "Whether new submissions (see 'submissions') are accepted at the moment"
        canSubmit: Boolean!
    }
`;


export class ContestProblemAssignmentUserTackling {
    constructor(readonly assignment: ContestProblemAssignment, readonly user: User) {}
    __typename = 'ContestProblemAssignmentUserTackling';
    async canSubmit({}, ctx: ApiContext) {
        return ctx.api(ContestProblemAssignmentUserTacklingApi).canSubmit(this);
    }
    async submissions({}, ctx: ApiContext) {
        return ctx.api(SubmissionApi).allByTackling.load(this);
    }
    assignmentView() {
        return new ContestProblemAssignmentView(this.assignment, this.user);
    }
}

export interface ContestProblemAssignmentUserTacklingModelRecord {
    ContestProblemAssignmentUserTackling: ContestProblemAssignmentUserTackling;
}

export class ContestProblemAssignmentUserTacklingApi extends ApiObject {
    async canSubmit(t: ContestProblemAssignmentUserTackling) {
        const status = await this.ctx.api(ContestApi).getStatus(t.assignment.problem.contest);

        return status === 'RUNNING';
    }

    async getAwardTacklings({ assignment, user }: ContestProblemAssignmentUserTackling) {
        const material = await this.ctx.api(ProblemMaterialApi).dataLoader.load(assignment.problem);

        return material.awards.map(
            award => new ContestAwardAssignmentUserTackling(new ContestAwardAssignment(assignment, award), user),
        );
    }

    async getScoreGrade(t: ContestProblemAssignmentUserTackling) {
        const awardTacklings = await this.getAwardTacklings(t);
        const awardGrades = await Promise.all(
            awardTacklings.map(t2 => this.ctx.api(ContestAwardAssignmentUserTacklingApi).getGrade(t2)),
        );

        return ScoreGrade.total(awardGrades.filter((g): g is ScoreGrade => g instanceof ScoreGrade));
    }
}
