import Table from "sap/m/Table";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import JSONModel from "sap/ui/model/json/JSONModel";
import { SearchField$SearchEvent } from "sap/m/SearchField";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import ListBinding from "sap/ui/model/ListBinding";
import MessageToast from "sap/m/MessageToast";
import Button, { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ScoringTable extends BaseController {

  private table: Table;
  private scoringModel: JSONModel;

  async onInit(): Promise<void> {
    super.getView()?.addStyleClass((super.getOwnerComponent() as MyComponent).getContentDensityClass());

    this.table = super.getView()?.byId("scoringTable") as Table;

    this.scoringModel = await super.createJSONModel(`/api/regattas/${super.getRegattaId()}/calculateScoring`, this.table);
    super.setViewModel(this.scoringModel, "scoring");

    super.getRouter()?.getRoute("scoring")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadScoringModel(), this);
  }

  onNavBack(): void {
    super.navBack("startpage");
  }

  onFilterSearch(event: SearchField$SearchEvent): void {
    const searchFilters: Filter[] = [];
    const query: string | undefined = event.getParameter("query")?.trim();
    if (query) {
      searchFilters.push(
        new Filter({
          filters: [
            new Filter("club/shortName", FilterOperator.Contains, query),
            new Filter("club/longName", FilterOperator.Contains, query),
            new Filter("club/city", FilterOperator.Contains, query),
            new Filter("club/abbreviation", FilterOperator.Contains, query)
          ],
          and: false
        }))
    }
    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding?.filter(searchFilters);
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    await this.loadScoringModel();
    MessageToast.show(this.i18n("msg.dataUpdated"));
    source.setEnabled(true);
  }

  private async loadScoringModel(): Promise<void> {
    await super.updateJSONModel(this.scoringModel, `/api/regattas/${super.getRegattaId()}/calculateScoring`, this.table)
  }

}