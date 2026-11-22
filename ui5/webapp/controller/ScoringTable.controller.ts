import Table from "sap/m/Table";
import BaseController from "./Base.controller";
import { Routes } from "../model/Routes";
import JSONModel from "sap/ui/model/json/JSONModel";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import ListBinding from "sap/ui/model/ListBinding";
import Button, { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";

/**
 * Club scoring table with text search and on-demand refresh for the active regatta.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ScoringTableController extends BaseController {
  private table?: Table;
  private readonly scoringModel: JSONModel = new JSONModel();

  /** Registers the scoring model and refreshes it when the scoring route matches. */
  onInit(): void {
    this.table = super.getView()?.byId("scoringTable") as Table;

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.scoringModel, "scoring");
    super
      .getRouter()
      ?.getRoute(Routes.Scoring)
      ?.attachMatched((_: Route$MatchedEvent) => {
        super.runAsync(this.loadScoringModel(), "Failed to load scoring:");
      }, this);
  }

  onNavBack(): void {
    super.navToStartPage();
  }

  /**
   * Applies or clears text-search constraints on the scoring table.
   *
   * @param event - Live-change event containing new search text; blank text clears search filters.
   */
  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = query ? this.createSearchFilters(query) : [];
    const binding: ListBinding | undefined = this.table?.getBinding("items") as ListBinding;
    binding?.filter(searchFilters);
  }

  /**
   * Reloads scoring and restores the triggering button even when loading rejects.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   */
  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    super.runAsync(
      this.loadScoringModel()
        .then((succeeded: boolean) => {
          super.showDataUpdatedMessage(succeeded);
        })
        .finally(() => source.setEnabled(true)),
      "Failed to refresh scoring:"
    );
  }

  /**
   * Builds an OR search across the scored club's names, city, and abbreviation.
   *
   * @param query - Trimmed search text applied to the scored club's fields.
   * @returns One compound OR filter combining all supported club search fields.
   */
  private createSearchFilters(query: string): Filter[] {
    return [
      new Filter({
        filters: [
          new Filter("club/shortName", FilterOperator.Contains, query),
          new Filter("club/longName", FilterOperator.Contains, query),
          new Filter("club/city", FilterOperator.Contains, query),
          new Filter("club/abbreviation", FilterOperator.Contains, query)
        ],
        and: false
      })
    ];
  }

  /**
   * Requests scoring for the active regatta.
   *
   * @returns Whether the scoring request succeeded; active-regatta loading failures still reject.
   */
  private async loadScoringModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    return await super.updateJSONModel(this.scoringModel, `/api/regattas/${regatta.id}/calculateScoring`);
  }
}
