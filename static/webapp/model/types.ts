/**
 * Type definitions for the Formatter class
 * @namespace de.regatta_hd.infoportal.model.types
 */

export interface Notification {
    id: number;
    severity: number;
    text: string;
}

export interface Race {
    number?: number;
    shortLabel?: string;
    comment?: string;
    cancelled?: boolean;
    state?: RaceState;
    distance?: number;
    heatsCount?: number;
    groupMode?: number;
}

export interface Heat {
    label?: string;
    dateTime?: string;
    cancelled?: boolean;
    state?: HeatState;
    roundCode?: RoundCode;
    groupValue?: number;
    race?: {
        ageClass?: {
            numSubClasses?: number;
        };
    };
}

export interface Entry {
    shortLabel?: string;
    cancelled?: boolean;
    groupValue?: number;
    boatNumber?: number;
    comment?: string;
    bib?: string;
}

export interface HeatEntry {
    entry: Entry;
    result?: {
        rankSort?: number;
    };
}

export interface Athlete {
    firstName: string;
    lastName: string;
    year: number;
    club: Club;
}

export interface Club {
    abbreviation?: string;
    shortName?: string;
    longName?: string;
}

export interface Crew {
    cox?: boolean;
    pos?: number;
    athlete: Athlete;
}

export enum RaceState {
    Initial = 0,
    Scheduled = 1,
    Started = 2,
    Unknown = 3,
    Official = 4,
    Finished = 5,
    PhotoFinish = 6,
}

export enum HeatState {
    Initial = 0,
    Seeded = 1,
    Started = 2,
    Official = 4,
    Finished = 5,
    PhotoFinish = 6,
}

export type RoundCode = "A" | "H" | "R" | "V" | "S" | "F";

export enum GroupValue {
    A = 0,
    B = 4,
    C = 8,
    D = 12,
    E = 16,
    F = 20,
    G = 24,
    H = 28,
    I = 32,
    J = 36,
}
