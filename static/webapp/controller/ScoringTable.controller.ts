import Table from "sap/m/Table";
import BaseController from "./Base.controller";
import JSONModel from "sap/ui/model/json/JSONModel";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import ListBinding from "sap/ui/model/ListBinding";
import Button, { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import MessageToast from "sap/m/MessageToast";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ScoringTable extends BaseController {

  private table: Table;
  private scoringModel: JSONModel = new JSONModel();

  onInit(): void {
    this.table = super.getView()?.byId("scoringTable") as Table;

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.scoringModel, "scoring");
    super.getRouter()?.getRoute("scoring")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadScoringModel(), this);

  }

  onNavBack(): void {
    super.navBack("startpage");
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = (query) ? this.createSearchFilters(query) : [];
    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding?.filter(searchFilters);
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    const updated: boolean = await this.loadScoringModel();
    if (updated) {
      MessageToast.show(this.i18n("msg.dataUpdated"));
    }
    source.setEnabled(true);
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter("club/shortName", FilterOperator.Contains, query),
        new Filter("club/longName", FilterOperator.Contains, query),
        new Filter("club/city", FilterOperator.Contains, query),
        new Filter("club/abbreviation", FilterOperator.Contains, query)
      ],
      and: false
    })]
  }

  private async loadScoringModel(): Promise<boolean> {
    return await super.updateJSONModel(this.scoringModel, `/api/regattas/${super.getRegattaId()}/calculateScoring`, this.table)
  }

}