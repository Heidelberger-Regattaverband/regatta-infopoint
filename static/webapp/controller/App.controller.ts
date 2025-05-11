import MyComponent from "de/regatta_hd/infoportal/Component";
import Controller from "sap/ui/core/mvc/Controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class AppController extends Controller {

  onInit(): void {
    super.getView()?.addStyleClass((this.getOwnerComponent() as MyComponent).getContentDensityClass());
  }

}