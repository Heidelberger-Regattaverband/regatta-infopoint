import type { RouteName } from "./Routes";

/**
 * Shared navigation state and the partial backend data shapes consumed by `Formatter`.
 * These interfaces describe frontend needs, not the complete REST API payloads.
 *
 * @namespace de.regatta_hd.infoportal.model.types
 */

/**
 * List-selection state shared with race and heat detail views.
 * Stored in the component's dedicated `raceNav` and `heatNav` models so UI metadata stays out of backend payloads.
 */
export interface NavigationData {
  /** Disables the first/previous buttons when the selected item is at the start of the list. */
  isFirst: boolean;
  /** Disables the next/last buttons when the selected item is at the end of the list. */
  isLast: boolean;
  /** Hides list-navigation buttons when details were opened without a list to traverse. */
  disabled: boolean;
  /** Fallback back-navigation route; defaults to the canonical list route when omitted. */
  back?: RouteName;
}

/** Race fields needed for labels, distance display, grouping, and state highlighting. */
export interface Race {
  number?: number;
  shortLabel?: string;
  comment?: string;
  cancelled?: boolean;
  state?: RaceState;
  distance?: number;
  /** Number of heats; used with the heat collection to determine whether any heats can be displayed. */
  heatsCount?: number;
  /** Backend grouping mode; mode 2 adds age-group and comment information to boat labels. */
  groupMode?: number;
}

/** Heat fields needed for schedule labels, round labels, and state highlighting. */
export interface Heat {
  label?: string;
  /** Backend date-time string used for the weekday and scheduled start time. */
  dateTime?: string;
  cancelled?: boolean;
  state?: HeatState;
  roundCode?: RoundCode;
  /** Encoded age-group value; see {@link GroupValue}. */
  groupValue?: number;
  race?: {
    ageClass?: {
      /** A positive value enables age-subclass information in the heat label. */
      numSubClasses?: number;
    };
  };
}

/** Registered boat entry fields used in labels and cancellation highlighting. */
export interface Entry {
  shortLabel?: string;
  cancelled?: boolean;
  groupValue?: number;
  boatNumber?: number;
  comment?: string;
  /** Bib identifier shown before the boat label when present. */
  bib?: string;
}

/** Entry in a heat with an optional result used to choose its highlight color. */
export interface HeatEntry {
  entry: Entry;
  result?: {
    /** Backend result order; values 1 through 5 receive the successful-result highlight. */
    rankSort?: number;
  };
}

/** Athlete identity and club information required to build a crew label. */
export interface Athlete {
  firstName: string;
  lastName: string;
  /** Birth year, displayed alongside the athlete's name. */
  year: number;
  club: Club;
}

/** Club names, preferred in abbreviation, short-name, then long-name order for athlete labels. */
export interface Club {
  abbreviation?: string;
  shortName?: string;
  longName?: string;
}

/** Athlete's position in a boat; coxswains are labeled `St` instead of a seat number. */
export interface Crew {
  cox?: boolean;
  pos?: number;
  athlete: Athlete;
}

/** Aquarius race-state codes, preserved verbatim for backend compatibility. */
export enum RaceState {
  Initial = 0,
  Scheduled = 1,
  Started = 2,
  Unknown = 3,
  Official = 4,
  Finished = 5,
  PhotoFinish = 6
}

/** Aquarius heat-state codes; the absence of value 3 is intentional. */
export enum HeatState {
  Initial = 0,
  Seeded = 1,
  Started = 2,
  Official = 4,
  Finished = 5,
  PhotoFinish = 6
}

/** Aquarius round codes: division, repechage, main race, forerun, semifinal, and final. */
export type RoundCode = "A" | "H" | "R" | "V" | "S" | "F";

/** Encoded age-group values mapped to display labels `AK A` through `AK J`. */
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
  J = 36
}
