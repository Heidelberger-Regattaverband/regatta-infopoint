/**
 * Type definitions for the Formatter class
 * @namespace de.regatta_hd.infoportal.model.types
 */

/**
 * Navigation state shared between a list controller (`RacesTable` /
 * `HeatsTable`) and the corresponding detail view (`RaceDetails` /
 * `HeatDetails`). Lives in the dedicated `raceNav` / `heatNav` JSONModels on
 * the Component (cf. review issue #4) so backend payloads are never mutated
 * with UI metadata.
 *
 * The XML bindings consume the fields like so:
 * ```
 * enabled="{=!${raceNav>/isFirst}}" visible="{=!${raceNav>/disabled}}"
 * ```
 *
 * @property isFirst  `true` when the current item is the first in the list —
 *                    used to disable the *first* and *previous* buttons.
 * @property isLast   `true` when the current item is the last in the list —
 *                    used to disable the *next* and *last* buttons.
 * @property disabled Hides all four nav buttons (e.g. on a deep-link
 *                    navigation where there is no list to traverse, or when
 *                    the detail view was opened from an unrelated context such
 *                    as the athletes / clubs detail screens).
 * @property back     Optional route name to navigate to when the user presses
 *                    *back*. When unset, the controller falls back to the
 *                    canonical list route (`races` / `heats`).
 */
export interface NavigationData {
    isFirst: boolean;
    isLast: boolean;
    disabled: boolean;
    back?: string;
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
