import { gql } from 'apollo-server-core';
import * as fs from 'fs';
import { DateTime } from 'luxon';
import * as path from 'path';
import {
    Column,
    HasMany,
    Index,
    Model,
    Table,
    Unique,
} from 'sequelize-typescript';
import { Resolvers } from '../generated/graphql-types';
import { ApiContext } from '../main/context';
import { ContestProblem } from './contest-problem';
import { FileContent } from './file-content';
import { ProblemFile } from './problem-file';

export const problemSchema = gql`
    type Problem {
        name: ID!
        files: [ProblemFile!]!
    }

    input ProblemInput {
        name: ID!
        files: [ID!]!
    }
`;

/** A problem in TuringArena. */
@Table
export class Problem extends Model<Problem> {
    /** Name of the problem, must be a valid identifier. */
    @Unique
    @Column
    @Index
    name!: string;

    /** Contests that contains this problem */
    @HasMany(() => ContestProblem)
    contestProblems: ContestProblem[];

    /** Files that belongs to this problem. */
    @HasMany(() => ProblemFile)
    files: ProblemFile[];
    getFiles: (options: object) => Promise<ProblemFile[]>;
    findFile: (options: object) => Promise<ProblemFile>;
    createFile: (file: object, options?: object) => Promise<ProblemFile>;

    /**
     * Extract the files of this problem in the specified base dir:
     * ${base}/${this.name}/${this.updatedAt}/<files...>
     * The last updated timestamp of this problem is appended, and
     * nothing is done if the directory already exists.
     * Creates all the directories if they don't exist.
     *
     * @param ctx Context to use
     * @param base Base directory
     */
    async extract(ctx: ApiContext, base: string) {
        const problemDir = path.join(
            base,
            this.name,
            DateTime.fromJSDate(this.updatedAt).toFormat(
                'x--yyyy-MM-dd--hh-mm-ss',
            ),
        );

        try {
            if ((await fs.promises.stat(problemDir)).isDirectory())
                return problemDir;
        } catch {
            // Directory doesn't exist and thus stat fails
        }

        const problemFiles = await this.getFiles({
            include: [ctx.db.FileContent],
        });

        for (const problemFile of problemFiles) {
            const filePath = path.join(problemDir, problemFile.path);
            await problemFile.content.extract(filePath);
        }

        return problemDir;
    }

    /**
     * Import the problem files from the filesystem
     *
     * @param ctx  Context to use
     * @param base Base directory to add
     * @param dir  Current directory
     */
    async loadFiles(ctx, base: string, dir: string = '') {
        const files = fs.readdirSync(path.join(base, dir));
        console.log('ADD FILES');
        console.log({ base, dir, files });
        for (const file of files) {
            const relPath = path.join(dir, file);
            if (fs.statSync(path.join(base, relPath)).isDirectory())
                await this.loadFiles(ctx, base, relPath);
            else {
                const content = await FileContent.createFromPath(
                    ctx,
                    path.join(base, relPath),
                );
                await this.createFile({
                    path: relPath,
                    contentId: content.id,
                });
            }
        }
    }

    async metadata(ctx: ApiContext): Promise<ProblemMetadata> {
        const metadataPath = '.task-info.json';
        const metadataProblemFile = await ctx.db.ProblemFile.findOne({
            where: {
                problemId: this.id,
                path: metadataPath,
            },
        });

        if (metadataProblemFile === null)
            throw new Error(
                `Problem ${this.name} is missing metadata file ${metadataPath}`,
            );

        const metadataFile = await metadataProblemFile.getContent();

        return JSON.parse(metadataFile.content.toString());
    }
}

/** Generic problem metadata */
interface ProblemMetadata {
    version: number;
    task_type: string;
    name: string;
    title: string;
    limits: {
        time: number;
        memory: number;
    };
    scoring: {
        max_score: number;
        subtasks: Array<{
            max_score: number;
            testcases: number;
        }>;
    };
    statements: Array<{
        language: string;
        content_type: string;
        path: string;
    }>;
    attachments: Array<{
        name: string;
        content_type: string;
        path: string;
    }>;
}

export const problemResolvers: Resolvers = {
    Problem: {
        files: problem => problem.getFiles(),
    },
};
