import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import Table from "sap/m/Table";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import { Routes } from "../model/Routes";
import { NavigationData } from "../model/types";
import BaseController from "./Base.controller";
import RacesTableController from "./RacesTable.controller";

/**
 * Displays an athlete and their race entries, with navigation to club and race details.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class AthleteDetailsController extends BaseController {
  private static readonly ATHLETE_MODEL: string = "athlete";
  private static readonly ENTRIES_MODEL: string = "entries";

  readonly formatter: Formatter = Formatter;
  private table?: Table;
  private athleteId?: number;

  /** Registers detail models and loads them when the athlete route matches. */
  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    this.table = super.getView()?.byId("athleteEntriesTable") as Table;

    super.setViewModel(new JSONModel(), AthleteDetailsController.ENTRIES_MODEL);
    super.setViewModel(new JSONModel(), AthleteDetailsController.ATHLETE_MODEL);

    super
      .getRouter()
      ?.getRoute(Routes.AthleteDetails)
      ?.attachPatternMatched((event: Route$PatternMatchedEvent) => {
        super.runAsync(this.onPatternMatched(event), "Failed to load athlete details:");
      }, this);
  }

  onNavBack(): void {
    super.navBack(Routes.Athletes);
    delete this.athleteId;
  }

  onClubPress(): void {
    const clubId: number | undefined = super
      .getViewJSONModel(AthleteDetailsController.ATHLETE_MODEL)
      .getProperty("/club/id");
    if (clubId !== undefined) {
      super.navToClubDetails(clubId);
    }
  }

  /**
   * Opens the selected entry's race without list-navigation controls and uses the athlete list as its back target.
   *
   * @param event - Selection event whose list item is bound to the athlete's race entry.
   */
  onSelectionChange(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(
        AthleteDetailsController.ENTRIES_MODEL
      );
      const entry: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      super.getComponentJSONModel(RacesTableController.RACE_MODEL).setData(entry.race);

      const navData: NavigationData = { isFirst: false, isLast: false, disabled: true, back: Routes.Athletes };
      super.getComponentJSONModel(RacesTableController.RACE_NAV_MODEL).setData(navData);

      super.navToRaceDetails(entry.race.id);
    }
  }

  /**
   * Refreshes both models and restores the triggering button even when loading rejects.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   */
  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    super.runAsync(
      this.loadData()
        .then((succeeded: [boolean, boolean]) => {
          super.showDataUpdatedMessage(succeeded[0] && succeeded[1]);
        })
        .finally(() => source.setEnabled(true)),
      "Failed to refresh athlete details:"
    );
  }

  /**
   * Captures the athlete ID before the first await because UI5 route-event parameters may be reused.
   *
   * @param event - Route match containing `athleteId` in its arguments.
   * @returns A promise resolving after both detail requests settle; regatta-lookup failures reject.
   */
  private async onPatternMatched(event: Route$PatternMatchedEvent): Promise<void> {
    this.athleteId = (event.getParameter("arguments") as any).athleteId;
    await this.loadData();
  }

  /**
   * Refreshes athlete details and race entries in parallel for the active regatta.
   *
   * @returns Update results in `[entries, athlete]` order.
   */
  private async loadData(): Promise<[boolean, boolean]> {
    const regatta: any = await super.getActiveRegatta();

    const athleteUrl: string = `/api/regattas/${regatta.id}/athletes/${this.athleteId}`;
    const entriesUrl: string = `/api/regattas/${regatta.id}/athletes/${this.athleteId}/entries`;

    const athleteModel: JSONModel = super.getViewJSONModel(AthleteDetailsController.ATHLETE_MODEL);
    const entriesModel: JSONModel = super.getViewJSONModel(AthleteDetailsController.ENTRIES_MODEL);

    return await Promise.all([
      super.updateJSONModel(entriesModel, entriesUrl),
      super.updateJSONModel(athleteModel, athleteUrl)
    ]);
  }
}
