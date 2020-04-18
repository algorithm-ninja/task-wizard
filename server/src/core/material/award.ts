import { gql } from 'apollo-server-core';
import { Resolvers } from '../../main/resolver-types';
import { FulfillmentGradeDomain } from '../feedback/fulfillment';
import { ScoreGradeDomain, ScoreRange } from '../feedback/score';
import { ProblemMaterial } from './problem-material';

export const awardSchema = gql`
    """
    Graded element in a problem.
    For every award of a problem, a progressively higher grade can be achieved during a contest.
    Corresponds to a subtask in IOI-like problems, assuming max-by-subtask scoring.
    """
    type Award {
        id: ID!

        problem: Problem!

        "Name used to identify this award in this problem. Only for admins."
        name: String!
        "Name of this award to display to users."
        title: Text!
        "Possible grades that can be achieved for this award"
        gradeDomain: GradeDomain!
    }
`;

export class Award {
    constructor(readonly material: ProblemMaterial, readonly index: number) {}

    private readonly subtaskInfo = this.material.taskInfo.IOI.scoring.subtasks[this.index];

    name = `subtask.${this.index}`;
    title = [{ value: `Subtask ${this.index}` }];

    gradeDomain =
        this.subtaskInfo.max_score > 0
            ? new ScoreGradeDomain(new ScoreRange(this.subtaskInfo.max_score, 0, true))
            : new FulfillmentGradeDomain();
}

export interface AwardModelRecord {
    Award: Award;
}

export const awardResolvers: Resolvers = {
    Award: {
        id: a => `${a.material.problem.contest.id}/${a.material.problem.name}/${a.index}`,
        problem: a => a.material.problem,
        name: a => a.name,
        gradeDomain: a => a.gradeDomain,
        title: a => a.title,
    },
};
